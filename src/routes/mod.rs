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

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    success: bool,
    message: Option<String>,
}

impl ApiResponse {
    pub fn success() -> Self {
        Self {
            success: true,
            message: None,
        }
    }

    pub fn failure(msg: String) -> Self {
        Self {
            success: false,
            message: Some(msg),
        }
    }
}
