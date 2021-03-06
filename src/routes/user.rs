use super::{
    super::{db::*, Uniques},
    ApiResponse,
    SessionInfo,
};
use actix_identity::Identity;
use actix_web::{error, web, HttpResponse, Result};
use futures::stream::TryStreamExt;
use mongodb::{
    bson::doc,
    error::{ErrorKind, WriteFailure},
};
use serde::Deserialize;
use std::ops::Deref;

use sha1::Digest;

const DUPLICATE_KEY_ERROR_CODE: i32 = 11000;

const DB: &str = "perpetual";
const USERS: &str = "users";
const RECIPIENTS: &str = "recipients";

async fn signup(
    client: web::Data<mongodb::Client>,
    uniques: web::Data<Uniques>,
    tmpl: web::Data<tera::Tera>,
    user_data: web::Form<UserProto>,
    web_client: web::Data<reqwest::Client>,
) -> Result<HttpResponse> {
    let mut ctx = tera::Context::new();
    ctx.insert("username", &user_data.username);
    ctx.insert("email", &user_data.email);
    ctx.insert("first_name", &user_data.first_name);
    ctx.insert("last_name", &user_data.last_name);

    let phone_number = user_data.phone.replace("-", "");
    if phone_number.len() != 10 || phone_number.chars().any(|c| !c.is_digit(10)) {
        ctx.insert("phone_error", "Invalid phone number");
        let s = tmpl.render("signup.html", &ctx).map_err(|e| {
            dbg!(&e);
            actix_web::error::ErrorInternalServerError("Template error")
        })?;

        return Ok(HttpResponse::Conflict().content_type("text/html").body(s));
    }

    if check_hibp(web_client.as_ref(), &user_data.password).await {
        return Ok(HttpResponse::PreconditionFailed().body("<h1>The password you entered has been <a href='https://www.troyhunt.com/introducing-306-million-freely-downloadable-pwned-passwords/'>compromised</a>.
        </h1><p>Perpetual recommends you use a <a href='https://1password.com'>password manager.</a>"));
    }

    let err = match user_data.clone().insert(client.get_ref(), DB, USERS).await {
        Ok(..) => {
            uniques.usernames.insert(user_data.username.clone());
            uniques.emails.insert(user_data.email.clone());

            return Ok(HttpResponse::Found()
                .append_header((actix_web::http::header::LOCATION, "/"))
                .finish());
        }
        Err(e) => e,
    };

    let s = match err.kind.as_ref() {
        ErrorKind::WriteError(WriteFailure::WriteError(write_error))
            if write_error.code == DUPLICATE_KEY_ERROR_CODE
                && write_error.message.contains("email") =>
        {
            ctx.insert("email_error", "That email already exists");
            tmpl.render("signup.html", &ctx).map_err(|e| {
                dbg!(&e);
                actix_web::error::ErrorInternalServerError("Template error")
            })?
        }
        ErrorKind::WriteError(WriteFailure::WriteError(write_error))
            if write_error.code == DUPLICATE_KEY_ERROR_CODE
                && write_error.message.contains("username") =>
        {
            ctx.insert("username_error", "That username already exists");
            tmpl.render("signup.html", &ctx).map_err(|e| {
                dbg!(&e);
                actix_web::error::ErrorInternalServerError("Template error")
            })?
        }
        _ => return Ok(HttpResponse::InternalServerError().body("Internal Server Error")),
    };
    Ok(HttpResponse::Conflict().content_type("text/html").body(s))
}

#[derive(Debug, Deserialize)]
struct LoginInfo {
    username: String,
    password: String,
}

async fn login(
    client: web::Data<mongodb::Client>,
    tmpl: web::Data<tera::Tera>,
    login_info: web::Form<LoginInfo>,
    identity: Identity,
) -> Result<HttpResponse> {
    let result = client
        .database(DB)
        .collection_with_type(USERS)
        .find_one(
            doc! {
                "username": &login_info.username,
            },
            None,
        )
        .await;

    let User { data, id } = match result {
        Ok(Some(user)) => user,
        Ok(None) => {
            let mut ctx = tera::Context::new();
            ctx.insert("badLogin", "No user found with these credentials.");
            let res = tmpl.render("login.html", &ctx).map_err(|e| {
                dbg!(&e);
                error::ErrorInternalServerError("Template error")
            })?;
            return Ok(HttpResponse::Ok().content_type("text/html").body(res));
        }
        Err(..) => {
            return Ok(
                HttpResponse::InternalServerError().body(format!("unable to connect to database"))
            )
        }
    };

    if !libpasta::verify_password(&data.password, &login_info.password) {
        return Ok(HttpResponse::Unauthorized().body("invalid username or password"));
    }

    if let Some(existing_login_user) = identity.identity() {
        dbg!(
            "logging out of {} to log into {}",
            existing_login_user,
            &data.username,
        );
        identity.forget();
    }

    let session = SessionInfo {
        username: data.username,
        id,
    };

    identity.remember(serde_json::to_string(&session).expect("serde unable to serialize session"));

    Ok(HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/"))
        .finish())
}

pub async fn user_home_page(
    client: web::Data<mongodb::Client>,
    tmpl: web::Data<tera::Tera>,
    identity: Identity,
) -> Result<HttpResponse> {
    let SessionInfo { username, id } = match identity.identity() {
        Some(identity) => {
            serde_json::from_str(&identity).expect("serde unable to deserialize session")
        }
        None => {
            return Ok(HttpResponse::Found()
                .append_header((actix_web::http::header::LOCATION, "/login"))
                .finish())
        }
    };

    let result = client
        .database(DB)
        .collection_with_type(USERS)
        .find_one(
            doc! {
                "_id": &id,
                "username": username,
            },
            None,
        )
        .await;

    let User { data, .. } = match result {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(HttpResponse::Found()
                .append_header((actix_web::http::header::LOCATION, "/login"))
                .finish())
        }
        Err(e) => {
            dbg!(e);
            return Ok(HttpResponse::InternalServerError().body("unable to connect to database"));
        }
    };

    let result = client
        .database(DB)
        .collection_with_type(RECIPIENTS)
        .find(doc! { "giftingUser": id }, None)
        .await;

    let stream = match result {
        Ok(stream) => stream,
        Err(e) => {
            dbg!(e);
            return Ok(HttpResponse::InternalServerError().body("unable to connect to database"));
        }
    };

    let recipients: Vec<RecipientProto> = match stream.try_collect().await {
        Ok(recipients) => recipients,
        Err(e) => {
            dbg!(e);
            return Ok(HttpResponse::InternalServerError().body("unable to connect to database"));
        }
    };

    let mut ctx = tera::Context::new();
    ctx.insert("username", &data.username);
    ctx.insert("email", &data.email);
    ctx.insert("first_name", &data.first_name);
    ctx.insert("last_name", &data.last_name);
    ctx.insert("recipients", &recipients);

    let s = tmpl.render("loggedin.html", &ctx).map_err(|e| {
        dbg!(&e);
        actix_web::error::ErrorInternalServerError("Template error")
    })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub async fn new_recipient(
    client: web::Data<mongodb::Client>,
    user_data: web::Json<RecipientProto>,
    identity: Identity,
) -> Result<HttpResponse> {
    if user_data.first_name.is_empty()
        || user_data.last_name.is_empty()
        || user_data.address.is_empty()
        || user_data.gift_date.is_empty()
    {
        return Ok(HttpResponse::BadRequest().json(&ApiResponse::failure(
            "invalid input, fields cannot be blank".into(),
        )));
    }

    let SessionInfo { id, .. } = match identity.identity() {
        Some(identity) => {
            serde_json::from_str(&identity).expect("serde unable to deserialize session")
        }
        None => {
            return Ok(HttpResponse::Unauthorized().json(&ApiResponse::failure(
                "cannot add recipient without being logged in".into(),
            )));
        }
    };

    let result = user_data
        .clone()
        .insert(id, client.get_ref(), DB, RECIPIENTS)
        .await;

    match result {
        Ok(..) => Ok(HttpResponse::Ok().json(&ApiResponse::success())),
        Err(e) => {
            dbg!(e);
            return Ok(
                HttpResponse::InternalServerError().json(&ApiResponse::failure(
                    "unable to connect to database".into(),
                )),
            );
        }
    }
}

pub async fn delete_recipient(
    identity: Identity,
    client: web::Data<mongodb::Client>,
    user_data: web::Json<RecipientProto>,
) -> Result<HttpResponse> {
    let SessionInfo { id, .. } = match identity.identity() {
        Some(id) => serde_json::from_str(&id).expect("serde unable to deserialize session"),
        None => {
            return Ok(HttpResponse::Unauthorized().json(&ApiResponse::failure(
                "cannot delete recipient without being logged in".into(),
            )))
        }
    };

    let result = client
        .database(DB)
        .collection(RECIPIENTS)
        .delete_one(
            doc! {
                "firstName": &user_data.first_name,
                "lastName": &user_data.last_name,
                "address": &user_data.address,
                "giftDate": &user_data.gift_date,
                "giftingUser": id,
            },
            None,
        )
        .await;

    match result {
        Ok(delete_result) if delete_result.deleted_count == 0 => {
            dbg!(&delete_result);
            return Ok(HttpResponse::BadRequest().json(&ApiResponse::failure(
                "could not find recipient to delete".into(),
            )));
        }
        Ok(..) => Ok(HttpResponse::Ok().json(&ApiResponse::success())),
        Err(e) => {
            dbg!(e);
            return Ok(
                HttpResponse::InternalServerError().json(&ApiResponse::failure(
                    "unable to connect to database".into(),
                )),
            );
        }
    }
}

pub async fn logout(id: Identity) -> Result<HttpResponse> {
    id.forget();

    Ok(HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/"))
        .finish())
}

async fn check_hibp(web_client: impl Deref<Target = reqwest::Client>, pw: &str) -> bool {
    let mut hasher = sha1::Sha1::new();
    hasher.update(pw.as_bytes());
    let result = hasher.finalize();
    let hashed_pw = hex::encode_upper(result);
    let prefix = &hashed_pw[0..5];
    let hibp = web_client
        .get(&format!("https://api.pwnedpasswords.com/range/{}", prefix))
        .header("hibp-api-key", env!("HIBP_API_KEY"))
        .header("user-agent", "perpetual")
        .send()
        .await;
    let text = match hibp {
        Ok(res) => res.text().await,
        Err(_) => return false,
    };
    match text {
        Ok(body) => body
            .split("\r\n")
            .map(|s| Some(s.split(':').next()?))
            .find(|pw| pw.map(|s| format!("{}{}", prefix, s)).as_ref() == Some(&hashed_pw))
            .is_some(),
        Err(_) => false,
    }
}

pub fn user_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/user")
            .service(web::resource("/login").route(web::post().to(login)))
            .service(web::resource("/signup").route(web::post().to(signup)))
            .service(web::resource("/add-recipient").route(web::post().to(new_recipient)))
            .service(web::resource("/delete-recipient").route(web::post().to(delete_recipient)))
            .service(web::resource("/logout").route(web::post().to(logout)))
            .service(web::resource("").route(web::get().to(user_home_page))),
    );
}

#[cfg(test)]
mod tests {
    use super::check_hibp;
    use rand::Rng;

    #[tokio::test]
    async fn pwned_pw() {
        let web_client = reqwest::Client::new();
        assert!(check_hibp(&web_client, "hunter2").await);
    }

    #[tokio::test]
    async fn non_pwned_pw() {
        let pw: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let web_client = reqwest::Client::new();
        assert!(!check_hibp(&web_client, &pw).await);
    }
}
