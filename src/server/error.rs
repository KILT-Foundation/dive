use actix_web::error::ResponseError;
use std::sync::PoisonError;

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Sync Error")]
    Sync,
    #[error("Unknown")]
    Unknown,
}

impl<T> From<PoisonError<T>> for ServerError {
    fn from(_err: PoisonError<T>) -> Self {
        Self::Sync
    }
}

impl ResponseError for ServerError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        todo!()
    }

    fn status_code(&self) -> reqwest::StatusCode {
        todo!()
    }
}
