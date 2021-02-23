use super::super::{db::*, errors::ConflictError};
use actix_web::{web, HttpResponse, Result};
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
        Ok(res) => return Ok(HttpResponse::Ok().json(&res)),
        Err(e) => e,
    };
    let mut ctx = tera::Context::new();
    ctx.insert("username", &user_data.username);
    ctx.insert("email", &user_data.email);
    ctx.insert("first_name", &user_data.first_name);
    ctx.insert("last_name", &user_data.last_name);

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
        } // _ => Err(actix_web::error::ErrorInternalServerError(err)),
        _ => return Ok(HttpResponse::InternalServerError().body("Internal Server Error")),
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub fn user_config(config: &mut web::ServiceConfig) {
    config.service(web::scope("/user").route("/new", web::post().to(new)));
}
