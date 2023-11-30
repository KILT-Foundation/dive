use crate::{device::error::DeviceError, kilt::error::TxError};

/// Enum which represents all possible application errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Transaction error: {0}")]
    Tx(#[from] TxError),
    #[error("Device error: {0}")]
    Device(#[from] DeviceError),
}
