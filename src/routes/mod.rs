use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

mod base;
mod user;

pub use base::*;
pub use user::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionInfo {
    username: String,
    id: ObjectId,
}
