use actix_files as fs;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    body::Body,
    dev::{Server, ServiceResponse},
    http::StatusCode,
    middleware::{ErrorHandlerResponse, ErrorHandlers, Logger},
    web,
    App,
    HttpResponse,
    HttpServer,
    Result,
};

use serde::{Deserialize, Serialize};

use dashmap::DashSet;
use mongodb::Client;
use std::{env, net::TcpListener};
use tera::Tera;

mod db;
mod routes;

extern crate log;

// Custom error handlers, to return HTML responses when an error occurs.
fn error_handlers() -> ErrorHandlers<Body> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found(res: ServiceResponse) -> Result<ErrorHandlerResponse<Body>> {
    let response = get_error_response(&res, "Page not found fool");
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.into_body()),
    ))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> HttpResponse<Body> {
    let request = res.request();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |e: &str| {
        HttpResponse::build(res.status())
            .content_type("text/plain")
            .body(e.to_string())
    };

    let tera = request.app_data::<web::Data<Tera>>().map(|t| t.get_ref());
    match tera {
        Some(tera) => {
            let mut context = tera::Context::new();
            context.insert("error", error);
            context.insert("status_code", res.status().as_str());
            let body = tera.render("error.html", &context);

            match body {
                Ok(body) => HttpResponse::build(res.status())
                    .content_type("text/html")
                    .body(body),
                Err(e) => {
                    dbg!("falling back: {:?}", e);
                    fallback(error)
                }
            }
        }
        None => fallback(error),
    }
}

#[derive(Debug, Clone)]
pub struct Uniques {
    pub usernames: DashSet<String>,
    pub emails: DashSet<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAndEmail {
    pub username: String,
    pub email: String,
}
pub fn run(
    listener: TcpListener,
    client: Client,
    private_key: &'static str,
    uniques: Uniques,
) -> std::io::Result<Server> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();

    let reqwest_client = reqwest::Client::new();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(private_key.as_bytes())
                    .name("perpetual")
                    .secure(false),
            ))
            .wrap(error_handlers())
            .wrap(Logger::default())
            .data(client.clone())
            .data(tera.clone())
            .data(uniques.clone())
            .data(reqwest_client.clone())
            .service(
                fs::Files::new("/static", concat!(env!("CARGO_MANIFEST_DIR"), "/static"))
                    .show_files_listing(),
            )
            .configure(routes::user_config)
            .configure(routes::base_config)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
