use actix_web::{dev::HttpResponseBuilder, error, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum ConflictError {
    #[display(fmt = "That {} is already taken.", field)]
    ValidationError { field: String },
}

impl error::ResponseError for ConflictError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code()).body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            ConflictError::ValidationError { .. } => StatusCode::CONFLICT,
        }
    }
}
