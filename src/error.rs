use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};

use crate::{
    device::DeviceError,
    kilt::error::{DidError, TxError},
};

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Device error: {0}")]
    Device(#[from] DeviceError),
    #[error("Tx error: {0}")]
    Tx(#[from] TxError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HTTP Client: {0}")]
    HttpClient(#[from] reqwest::Error),
    #[error("HTTP Client: {0}")]
    HttpClientHeader(#[from] reqwest::header::InvalidHeaderValue),
    #[error("URL Error: {0}")]
    URL(#[from] url::ParseError),
    #[error("Login error: {0}")]
    Login(&'static str),
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
