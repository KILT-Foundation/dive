use super::crypto::Error as ZKError;
use bip39::Error as Bip39Error;
use serde_json::Error as JSONError;
use subxt::ext::sp_core::crypto::SecretStringError;

#[derive(thiserror::Error, Debug)]
pub enum DeviceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error generating random bytes")]
    Random,
    #[error("Mnemonic error: {0}")]
    Mnemonic(#[from] Bip39Error),
    #[error("JSON error: {0}")]
    JSON(#[from] JSONError),
    #[error("Secret String error: {0}")]
    Secret(#[from] SecretStringError),
    #[error("ZK error: {0}")]
    ZK(#[from] ZKError),
}
