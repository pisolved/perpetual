use std::net::TcpListener;

use perpetual::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("could not bind to a port");
    let uri = env!("MONGODB_URI");

    let client = mongodb::Client::with_uri_str(&uri)
        .await
        .expect("couldn't connect to mongodb");

    let private_key = env!("PERPETUAL_PRIVATE_KEY");

    run(listener, client, private_key)?.await
}
