// store tera template in application state

use actix_web::{
    body::Body,
    dev::ServiceResponse,
    error,
    middleware::{ErrorHandlerResponse, ErrorHandlers},
    web, Error, HttpResponse, Result,
};
use tera::Tera;
pub async fn index(tmpl: web::Data<Tera>) -> Result<HttpResponse, Error> {
    let res = tmpl
        .render("index.html", &tera::Context::new())
        .map_err(|e| {
            dbg!(&e);
            return error::ErrorInternalServerError("Template error");
        })?;
    Ok(HttpResponse::Ok().content_type("text/html").body(res))
}
