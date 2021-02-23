use super::{super::db::*, base::login_page};
use actix_identity::Identity;
use actix_web::{web, HttpResponse, Result};
use libpasta::verify_password;
use mongodb::{
    bson::doc,
    error::{ErrorKind, WriteFailure},
};
use serde::Deserialize;

const DUPLICATE_KEY_ERROR_CODE: i32 = 11000;

async fn new(
    client: web::Data<mongodb::Client>,
    tmpl: web::Data<tera::Tera>,
    user_data: web::Form<UserProto>,
) -> Result<HttpResponse> {
    let mut ctx = tera::Context::new();
    ctx.insert("username", &user_data.username);
    ctx.insert("email", &user_data.email);
    ctx.insert("first_name", &user_data.first_name);
    ctx.insert("last_name", &user_data.last_name);

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

#[derive(Debug, Deserialize)]
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
    let result = client
        .database("perpetual")
        .collection_with_type("users")
        .find_one(
            doc! {
                "username": &login_info.username,
            },
            None,
        )
        .await;

    let User { data, .. } = match result {
        Ok(Some(user)) => user,
        Ok(None) => return Ok(HttpResponse::Unauthorized().body("invalid username or password")),
        Err(..) => {
            return Ok(HttpResponse::InternalServerError().body("unable to connect to database"))
        }
    };

    if !libpasta::verify_password(&data.password, &login_info.password) {
        return Ok(HttpResponse::Unauthorized().body("invalid username or password"));
    }

    let mut ctx = tera::Context::new();
    ctx.insert("username", &data.username);
    ctx.insert("email", &data.email);
    ctx.insert("first_name", &data.first_name);
    ctx.insert("last_name", &data.last_name);

    if let Some(existing_login_user) = id.identity() {
        dbg!(
            "logging out of {} to log into {}",
            existing_login_user,
            &data.username,
        );
        id.forget();
        id.remember(data.username);
    }

    let s = tmpl.render("loggedin.html", &ctx).map_err(|e| {
        dbg!(&e);
        actix_web::error::ErrorInternalServerError("Template error")
    })?;

    return Ok(HttpResponse::Ok().content_type("text/html").body(s));
}

pub fn user_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/user")
            .service(web::resource("/login").route(web::post().to(login)))
            .service(web::resource("/new").route(web::post().to(new))),
    );
}
