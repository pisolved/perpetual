use mongodb::Client;

#[actix_rt::test]
async fn mongo_connection_works() {
    let uri = env!("MONGODB_URI");
    // let uri = env::var("MONGODB_URI").expect("no mongodb uri");
    let client = Client::with_uri_str(&uri)
        .await
        .expect("couldn't connect to mongodb");
    let dbs = client
        .list_database_names(None, None)
        .await
        .expect("couldn't list database names");
    assert!(&dbs.contains(&"develop".into()));
}
