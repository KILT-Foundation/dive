use std::{string::FromUtf8Error, sync::PoisonError};

use actix_web::error::PayloadError;

pub enum Error {
    Unknown,
    Io(std::io::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Unknown => write!(f, "unknown error"),
            Self::Io(ref err) => write!(f, "io error: {}", err),
            Self::Http(ref err) => write!(f, "error in request {}", err),
            Self::Json(ref err) => write!(f, "error in json {}", err),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for Error {}

impl From<Error> for actix_web::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Unknown => actix_web::error::ErrorInternalServerError(err),
            Error::Io(err) => actix_web::error::ErrorInternalServerError(err),
            Error::Http(err) => actix_web::error::ErrorInternalServerError(err),
            Error::Json(err) => actix_web::error::ErrorInternalServerError(err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(_err: Box<dyn std::error::Error>) -> Self {
        Self::Unknown
    }
}

impl From<subxt::Error> for Error {
    fn from(_err: subxt::Error) -> Self {
        Self::Unknown
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_err: PoisonError<T>) -> Self {
        Self::Unknown
    }
}

impl From<PayloadError> for Error {
    fn from(_err: PayloadError) -> Self {
        Self::Unknown
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_err: FromUtf8Error) -> Self {
        Self::Unknown
    }
}

impl From<hex::FromHexError> for Error {
    fn from(_err: hex::FromHexError) -> Self {
        Self::Unknown
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
