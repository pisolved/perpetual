mod helpers;
use helpers::spawn_app;
use std::collections::HashMap;

#[actix_rt::test]
async fn basic_db_ops() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let mut map = HashMap::new();
    map.insert("lang", "rust");

    // this route should insert a document, delete it, and return the number of
    // deleted documents in the body

    let response = client
        .post(&format!("http://{}/{}", &addr, "test/lang"))
        .json(&map)
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());

    let text = response
        .text()
        .await
        .expect("couldn't handle response text");
    assert_eq!(text, "1");
}
