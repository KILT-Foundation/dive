use hex::FromHexError;
use std::string::FromUtf8Error;

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
    #[error("DID has a wrong format : {0}")]
    Format(String),
}

#[derive(thiserror::Error, Debug)]
pub enum FormatError {
    #[error("UTF-8 decoding error: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("Hex error: {0}")]
    Hex(#[from] FromHexError),
}
