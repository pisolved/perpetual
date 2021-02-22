use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};

use super::super::db::*;

async fn health_check(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

async fn hello_name(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    format!("Hello, {}", name)
}

async fn insert_lang(
    client: web::Data<mongodb::Client>,
    item: web::Json<Lang>,
) -> Result<HttpResponse, Error> {
    match item.0.insert(client.get_ref()).await {
        Ok(res) => Ok(HttpResponse::Ok().body(res)),
        Err(e) => Err(actix_web::error::ErrorFailedDependency(e)),
    }
}

pub fn test_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/test")
            .route("/health_check", web::get().to(health_check))
            .route("/hello/{name}", web::get().to(hello_name))
            .route("/lang", web::post().to(insert_lang)),
    );
}
