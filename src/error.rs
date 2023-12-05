use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use std::sync::PoisonError;

use crate::{
    device::DeviceError,
    kilt::error::{DidError, TxError},
};

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Device error: {0}")]
    Device(#[from] DeviceError),
    #[error("Tx error: {0}")]
    Tx(#[from] TxError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Sync Error")]
    Sync,
    #[error("HTTP Client: {0}")]
    HttpClient(#[from] reqwest::Error),
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
        if self.status_code() != StatusCode::NOT_FOUND {
            log::error!("{}", self.to_string());
        }

        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    // TODO can be still be improved
    fn status_code(&self) -> StatusCode {
        match self {
            ServerError::Device(DeviceError::Io(ref io_err))
                if io_err.kind() == std::io::ErrorKind::NotFound =>
            {
                StatusCode::NOT_FOUND
            }
            ServerError::Tx(TxError::Did(DidError::NotFound(..))) => StatusCode::NOT_FOUND,
            ServerError::Json(..) | ServerError::Device(DeviceError::JSON(..)) => {
                StatusCode::BAD_REQUEST
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
