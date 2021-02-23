// store tera template in application state

use actix_web::{error, web, Error, HttpResponse, Result};
use tera::Tera;

pub async fn index(tmpl: web::Data<Tera>) -> Result<HttpResponse, Error> {
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

async fn signup(tmpl: web::Data<Tera>) -> Result<HttpResponse, Error> {
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

pub async fn login_page(tmpl: web::Data<Tera>) -> Result<HttpResponse, Error> {
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

pub fn base_config(config: &mut web::ServiceConfig) {
    config.service(web::scope("/signup").route("", web::get().to(signup)));
    config.service(web::scope("/login").route("", web::get().to(login_page)));
}
