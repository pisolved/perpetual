use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lang {
    lang: String,
}

impl Lang {
    pub async fn insert(&self, client: &Client) -> Result<String, mongodb::error::Error> {
        let coll: Collection<Lang> = client.database("develop").collection_with_type("buildfest");
        let inserted = coll.insert_one(self.clone(), None).await?;
        dbg!(&inserted);
        let deleted = coll
            .delete_one(doc! { "_id": inserted.inserted_id }, None)
            .await?;
        dbg!(&deleted);
        Ok(format!("{}", deleted.deleted_count))
    }
}
