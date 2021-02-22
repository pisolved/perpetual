use actix_files::Files;
use actix_web::{dev::Server, middleware, web, App, HttpServer};
use mongodb::Client;
use std::{env, net::TcpListener};

mod db;
mod routes;

extern crate log;

pub fn run(listener: TcpListener, client: Client) -> Result<Server, std::io::Error> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(client.clone())
            .data(web::JsonConfig::default())
            .configure(routes::user_config)
            .service(Files::new("/", "./src/static/root/").index_file("index.html"))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
