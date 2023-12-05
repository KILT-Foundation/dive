use std::string::FromUtf8Error;

use hex::FromHexError;

/// All possible errors while interacting with the Blockchain.
#[derive(thiserror::Error, Debug)]
pub enum TxError {
    #[error("Subxt error: {0}")]
    Subxt(#[from] subxt::Error),
    #[error("Format error: {0}")]
    Format(FormatError),
    #[error("Hex error: {0}")]
    Hex(#[from] hex::FromHexError),
    #[error("DID error: {0}")]
    Did(DidError),
}

#[derive(thiserror::Error, Debug)]
pub enum DidError {
    #[error("DID not found : {0}")]
    NotFound(String),
    #[error("Invalid key")]
    InvalidKey,
    #[error("Key missing ")]
    MissingKey,
    #[error("DID has a wrong format : {0}")]
    Format(String),
}

#[derive(thiserror::Error, Debug)]
pub enum FormatError {
    #[error("Could not convert. Key: {0}")]
    Convert(String),
    #[error("UTF-8 decoding error: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("Key URI error: {0}")]
    KeyUri(String),
    #[error("SS58 encoding error: {0}")]
    Ss58(String),
    #[error("Hex error: {0}")]
    Hex(#[from] FromHexError),
}
