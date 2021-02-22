use mongodb::{bson::doc, Client};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lang {
    lang: String,
}

impl Lang {
    pub async fn insert(self, client: &Client) -> Result<String, mongodb::error::Error> {
        let coll = client.database("develop").collection_with_type("buildfest");
        let inserted = coll.insert_one(self, None).await?;
        let deleted = coll
            .delete_one(doc! { "_id": inserted.inserted_id }, None)
            .await?;
        Ok(format!("{}", deleted.deleted_count))
    }
}
