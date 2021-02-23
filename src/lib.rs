use actix_session::CookieSession;
use actix_web::{
    body::Body,
    dev::{ResponseBody, Server, ServiceResponse},
    http::StatusCode,
    middleware::{ErrorHandlerResponse, ErrorHandlers, Logger},
    web,
    App,
    HttpResponse,
    HttpServer,
    Result,
};

use mongodb::Client;
use std::{env, net::TcpListener};
use tera::Tera;

mod db;
mod errors;
mod routes;

extern crate log;

// Custom error handlers, to return HTML responses when an error occurs.
fn error_handlers() -> ErrorHandlers<Body> {
    ErrorHandlers::new()
        .handler(StatusCode::NOT_FOUND, not_found)
        .handler(StatusCode::CONFLICT, conflict)
}

// Error handler for a 404 Page not found error.
fn not_found(res: ServiceResponse) -> Result<ErrorHandlerResponse<Body>> {
    let response = get_error_response(&res, "Page not found fool");
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.into_body()),
    ))
}

// Error handler for a 409 Conflict.
fn conflict(mut res: ServiceResponse) -> Result<ErrorHandlerResponse<Body>> {
    let body = match res.take_body() {
        ResponseBody::Body(body) => body,
        ResponseBody::Other(body) => body,
    };

    let msg = match body {
        Body::None | Body::Empty | Body::Message(..) => "Conflict".into(),
        Body::Bytes(bytes) => {
            if let Ok(s) = String::from_utf8(bytes.to_vec()) {
                s
            } else {
                "Conflict".into()
            }
        }
    };

    let response = get_error_response(&res, &msg);
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
pub fn run(listener: TcpListener, client: Client) -> Result<Server, std::io::Error> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let mut tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    tera.full_reload();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                CookieSession::signed(
                    "this is such a long string that it has 32 bytes in it (at least)".as_bytes(),
                )
                .secure(false),
            )
            .wrap(error_handlers())
            .wrap(Logger::default())
            .data(client.clone())
            .data(web::JsonConfig::default())
            .data(tera.clone())
            .configure(routes::user_config)
            .service(web::resource("/").route(web::get().to(routes::index)))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
