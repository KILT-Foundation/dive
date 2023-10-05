use std::sync::PoisonError;

use actix_web::error::PayloadError;

pub enum Error {
    Unknown,
    Io(std::io::Error),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Unknown => write!(f, "unknown error"),
            Self::Io(ref err) => write!(f, "io error: {}", err),
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

