use mongodb::{
    bson::{doc, oid::ObjectId},
    Client,
};
use serde::{Deserialize, Serialize};

use gregorian::Date;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Recipient {
    pub gifting_user: ObjectId,
    #[serde(flatten)]
    pub data: RecipientProto,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipientProto {
    pub first_name: String,
    pub last_name: String,
    pub gift_date: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GregorianDate {
    month: i32,
    day: i32,
}

impl From<Date> for GregorianDate {
    fn from(date: Date) -> Self {
        Self {
            month: date.month().to_number() as i32,
            day: date.day() as i32,
        }
    }
}

impl RecipientProto {
    pub async fn insert(
        self,
        gifting_user: ObjectId,
        client: &Client,
        db: &str,
        coll: &str,
    ) -> mongodb::error::Result<()> {
        let recipient = Recipient {
            gifting_user,
            data: self,
        };

        let coll = client.database(db).collection_with_type(coll);
        coll.insert_one(recipient.clone(), None).await?;
        Ok(())
    }
}
