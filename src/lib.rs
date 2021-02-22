use actix_web::{
    dev::Server, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use mongodb::Client;
use std::{env, net::TcpListener};

mod db;
use db::*;

extern crate log;

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

fn app_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("")
            .route("/health_check", web::get().to(health_check))
            .route("/hello/{name}", web::get().to(hello_name))
            .route("/test/lang", web::post().to(insert_lang)),
    );
}

pub fn run(listener: TcpListener, client: Client) -> Result<Server, std::io::Error> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(client.clone())
            .data(web::JsonConfig::default())
            .configure(app_config)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
