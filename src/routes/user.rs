use super::super::db::*;
use actix_web::{web, HttpResponse, Result};
use mongodb::error::{ErrorKind, WriteFailure};

const DUPLICATE_KEY_ERROR_CODE: i32 = 11000;

async fn new(
    client: web::Data<mongodb::Client>,
    user_data: web::Json<UserProto>,
) -> Result<HttpResponse> {
    let err = match user_data
        .clone()
        .insert(client.get_ref(), "perpetual", "users")
        .await
    {
        Ok(res) => return Ok(HttpResponse::Ok().json(&res)),
        Err(e) => e,
    };

    match err.kind.as_ref() {
        ErrorKind::WriteError(WriteFailure::WriteError(write_error))
            if write_error.code == DUPLICATE_KEY_ERROR_CODE
                && write_error.message.contains("email") =>
        {
            Err(actix_web::error::ErrorConflict(format!(
                "email {} already exists",
                user_data.email
            )))
        }
        ErrorKind::WriteError(WriteFailure::WriteError(write_error))
            if write_error.code == DUPLICATE_KEY_ERROR_CODE
                && write_error.message.contains("username") =>
        {
            Err(actix_web::error::ErrorConflict(format!(
                "username {} already exists",
                user_data.username
            )))
        }
        _ => Err(actix_web::error::ErrorInternalServerError(err)),
    }
}

pub fn user_config(config: &mut web::ServiceConfig) {
    config.service(web::scope("/user").route("/new", web::post().to(new)));
}
