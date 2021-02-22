use std::net::TcpListener;

use zero2prod::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("could not bind to a port");
    let uri = env!("MONGODB_URI");
    // let uri = env::var("MONGODB_URI").expect("no mongodb uri");
    let client = mongodb::Client::with_uri_str(&uri)
        .await
        .expect("couldn't connect to mongodb");

    run(listener, client)?.await
}
