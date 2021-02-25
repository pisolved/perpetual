use core::future;
use std::net::TcpListener;

use dashmap::DashSet;
use futures::TryStreamExt;
use mongodb::{bson::doc, options::FindOptions};

use perpetual::{run, Uniques, UserAndEmail};

const DB: &str = "perpetual";
const USERS: &str = "users";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("could not bind to a port");
    let uri = env!("MONGODB_URI");

    let client = mongodb::Client::with_uri_str(&uri)
        .await
        .expect("couldn't connect to mongodb");

    let uniques = get_usernames_and_emails(&client).await;

    let private_key = env!("PERPETUAL_PRIVATE_KEY");

    run(listener, client, private_key, uniques)?.await
}

async fn get_usernames_and_emails(client: &mongodb::Client) -> Uniques {
    let usernames = DashSet::<String>::new();
    let emails = DashSet::<String>::new();

    client
        .database(DB)
        .collection_with_type::<UserAndEmail>(USERS)
        .find(
            None,
            FindOptions::builder()
                .projection(doc! {"username": 1, "email": 1, "_id": 0})
                .build(),
        )
        .await
        .expect("Internal error with MongoDB")
        .try_for_each(|elem| {
            usernames.insert(elem.username);
            emails.insert(elem.email);
            future::ready(Ok(()))
        })
        .await
        .expect("Internal error with MongoDB");

    Uniques { usernames, emails }
}
