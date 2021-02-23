use super::super::db::*;
use super::base::login_page;
use actix_web::{web, HttpResponse, Identity, Result};
use mongodb::error::{ErrorKind, WriteFailure};

const DUPLICATE_KEY_ERROR_CODE: i32 = 11000;

async fn new(
    client: web::Data<mongodb::Client>,
    tmpl: web::Data<tera::Tera>,
    user_data: web::Form<UserProto>,
) -> Result<HttpResponse> {
   let err = match user_data
        .clone()
        .insert(client.get_ref(), "perpetual", "users")
        .await
    {
        Ok(..) => {
            return login_page(tmpl).await;
        }
        Err(e) => e,
    };

    let s = match err.kind.as_ref() {
        ErrorKind::WriteError(WriteFailure::WriteError(write_error))
            if write_error.code == DUPLICATE_KEY_ERROR_CODE
                && write_error.message.contains("email") =>
        {
            ctx.insert("email_error", "That email already exists");
            tmpl.render("index.html", &ctx).map_err(|e| {
                dbg!(&e);
                actix_web::error::ErrorInternalServerError("Template error")
            })?
        }
        ErrorKind::WriteError(WriteFailure::WriteError(write_error))
            if write_error.code == DUPLICATE_KEY_ERROR_CODE
                && write_error.message.contains("username") =>
        {
            ctx.insert("username_error", "That username already exists");
            tmpl.render("index.html", &ctx).map_err(|e| {
                dbg!(&e);
                actix_web::error::ErrorInternalServerError("Template error")
            })?
        }
        _ => return Ok(HttpResponse::InternalServerError().body("Internal Server Error")),
    };
    Ok(HttpResponse::Conflict().content_type("text/html").body(s))
}

#[derive(Debug)]
struct LoginInfo {
    username: String,
    password: String,
}

async fn login(
    client: web::Data<mongodb::Client>,
    tmpl: web::Data<tera::Tera>,
    login_info: web::Form<LoginInfo>,
    id: Identity,
) -> Result<HttpResponse> {
    let username = login_info.username;
    let password = libpasta::hash_password(&login_info.password);

    let result = client
        .database("perpetual")
        .collection("users")
        .find_one(
            doc! {
                "username": username,
                "password": password,
            },
            None).await;

    let User { data, .. } = match result {
        Ok(Some(user)) => user,
        Ok(None) => return Ok(HttpResponse::Unauthorized("invalid username or password"));
        Err(e) => Ok(HttpResponse::Conflict().content_type("text/html").body(e))
    };

    let mut ctx = tera::Context::new();
    ctx.insert("username", &username);
    ctx.insert("email", &email);
    ctx.insert("first_name", &first_name);
    ctx.insert("last_name", &last_name);

    if let Some(existing_login_user) = id.identity() {
        dbg!("logging out of {} to log into {}", existing_login_user, username);
        id.forget();
        id.remember(username);
    }

    let s = tmpl.render("loggedin.html", &ctx).map_err(|e| {
                dbg!(&e);
                actix_web::error::ErrorInternalServerError("Template error")
            })?;

            return Ok(HttpResponse::Ok().content_type("text/html").body(s));

}

pub fn user_config(config: &mut web::ServiceConfig) {
    config.service(web::scope("/user").route("/new", web::post().to(new)));
}
