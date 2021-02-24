// store tera template in application state

use actix_identity::Identity;
use actix_web::{error, web, Error, HttpResponse, Result};

use super::user_home_page;

pub async fn index(
    client: web::Data<mongodb::Client>,
    tmpl: web::Data<tera::Tera>,
    identity: Identity,
) -> Result<HttpResponse, Error> {
    match identity.identity() {
        Some(_) => user_home_page(client, tmpl, identity).await,
        None => {
            let mut ctx = tera::Context::new();
            ctx.insert("username", "");
            ctx.insert("email", "");
            ctx.insert("first_name", "");
            ctx.insert("last_name", "");

            let res = tmpl.render("index.html", &ctx).map_err(|e| {
                dbg!(&e);
                error::ErrorInternalServerError("Template error")
            })?;
            Ok(HttpResponse::Ok().content_type("text/html").body(res))
        }
    }
}

async fn signup(
    client: web::Data<mongodb::Client>,
    tmpl: web::Data<tera::Tera>,
    identity: Identity,
) -> Result<HttpResponse, Error> {
    match identity.identity() {
        Some(_) => user_home_page(client, tmpl, identity).await,
        None => {
            let mut ctx = tera::Context::new();
            ctx.insert("username", "");
            ctx.insert("email", "");
            ctx.insert("password", "");
            ctx.insert("first_name", "");
            ctx.insert("last_name", "");

            let res = tmpl.render("signup.html", &ctx).map_err(|e| {
                dbg!(&e);
                error::ErrorInternalServerError("Template error")
            })?;
            Ok(HttpResponse::Ok().content_type("text/html").body(res))
        }
    }
}

pub async fn login_page(
    client: web::Data<mongodb::Client>,
    tmpl: web::Data<tera::Tera>,
    identity: Identity,
) -> Result<HttpResponse, Error> {
    match identity.identity() {
        Some(_) => user_home_page(client, tmpl, identity).await,
        None => {
            let mut ctx = tera::Context::new();
            ctx.insert("username", "");
            ctx.insert("email", "");
            ctx.insert("password", "");
            ctx.insert("first_name", "");
            ctx.insert("last_name", "");

            let res = tmpl.render("login.html", &ctx).map_err(|e| {
                dbg!(&e);
                error::ErrorInternalServerError("Template error")
            })?;
            Ok(HttpResponse::Ok().content_type("text/html").body(res))
        }
    }
}

pub fn base_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/")
            .service(web::resource("signup").route(web::get().to(signup)))
            .service(web::resource("login").route(web::get().to(login_page)))
            .service(web::resource("").route(web::get().to(index))),
    );
}
