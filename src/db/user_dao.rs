use mongodb::{
    bson::{doc, oid::ObjectId},
    Client,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserProto {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(flatten)]
    pub data: UserProto,
}

impl UserProto {
    pub async fn insert(
        mut self,
        client: &Client,
        db: &str,
        coll: &str,
    ) -> mongodb::error::Result<User> {
        self.password = libpasta::hash_password(&self.password);

        let user = User {
            id: ObjectId::new(),
            data: self,
        };

        let coll = client.database(db).collection_with_type(coll);
        coll.insert_one(user.clone(), None).await?;
        Ok(user)
    }
}
