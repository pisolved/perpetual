use super::GregorianDate;
use gregorian::{Date, Month, Year};
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::UpdateOptions,
    Client,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserProto {
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
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
        self,
        client: &Client,
        db: &str,
        coll: &str,
    ) -> mongodb::error::Result<User> {
        let user = User {
            id: ObjectId::new(),
            data: self,
        };

        let coll = client.database(db).collection_with_type(coll);
        coll.insert_one(user.clone(), None).await?;
        Ok(user)
    }
}

impl User {
    pub async fn add_gift_date(
        &self,
        client: &Client,
        db: &str,
        coll: &str,
        first_name: &str,
        last_name: &str,
        address: &str,
        year: impl Into<Year>,
        month: Month,
        day: u8,
    ) -> anyhow::Result<()> {
        let year = year.into();

        let date = Date::new(year, month, day)?;
        let gregorian_date: GregorianDate = date.into();

        let coll = client.database(db).collection(coll);
        coll.update_one(
            doc! {
                "first_name": first_name,
                "last_name": last_name,
                "address": address,
                "gifting_user": &self.id,
            },
            doc! {
                "$addToSet": {
                    "giftDates": mongodb::bson::to_document(&gregorian_date)?,
                }
            },
            UpdateOptions::builder().upsert(true).build(),
        )
        .await?;

        Ok(())
    }
}
