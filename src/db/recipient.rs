use mongodb::{
    bson::{doc, oid::ObjectId},
    Client,
};
use serde::{Deserialize, Serialize};

use gregorian::Date;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Recipient {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(flatten)]
    pub data: RecipientProto,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipientProto {
    pub first_name: String,
    pub last_name: String,
    #[serde(default)]
    pub gift_dates: Vec<GregorianDate>,
    pub address: String,
    pub gifting_user: ObjectId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GregorianDate {
    year: i32,
    month: i32,
    day: i32,
}

impl From<Date> for GregorianDate {
    fn from(date: Date) -> Self {
        Self {
            year: date.year().to_number() as i32,
            month: date.month().to_number() as i32,
            day: date.day() as i32,
        }
    }
}

impl RecipientProto {
    pub async fn insert(self, client: &Client, db: &str, coll: &str) -> mongodb::error::Result<()> {
        let recipient = Recipient {
            id: ObjectId::new(),
            data: self,
        };

        let coll = client.database(db).collection_with_type(coll);
        coll.insert_one(recipient.clone(), None).await?;
        Ok(())
    }
}
