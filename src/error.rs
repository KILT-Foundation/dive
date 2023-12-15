use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use std::io::ErrorKind;

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

impl ResponseError for DeviceError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            DeviceError::Io(e) => {
                if e.kind() == ErrorKind::NotFound {
                    return StatusCode::NOT_FOUND;
                }
                StatusCode::INTERNAL_SERVER_ERROR
            }
            DeviceError::Random | DeviceError::ZK(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DeviceError::Mnemonic(_) | DeviceError::JSON(_) | DeviceError::Secret(_) => {
                StatusCode::BAD_REQUEST
            }
        }
    }
}

impl ResponseError for TxError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        if self.status_code() != StatusCode::INTERNAL_SERVER_ERROR {
            log::error!("{}", self.to_string());
        }
        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            TxError::Did(did_error) => match did_error {
                DidError::NotFound(_) => StatusCode::NOT_FOUND,
                DidError::Format(_) => StatusCode::BAD_REQUEST,
            },

            TxError::Subxt(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TxError::Format(_) | TxError::Hex(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl ResponseError for ServerError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            ServerError::Device(e) => e.error_response(),
            ServerError::Tx(e) => e.error_response(),
            _ => {
                log::error!("{}", self.to_string());
                HttpResponse::build(self.status_code()).body(self.to_string())
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ServerError::Device(e) => e.status_code(),
            ServerError::Tx(e) => e.status_code(),
            ServerError::Json(..) => StatusCode::BAD_REQUEST,
            ServerError::HttpClient(..)
            | ServerError::HttpClientHeader(..)
            | ServerError::URL(..)
            | ServerError::Login(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
