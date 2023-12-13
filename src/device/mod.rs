pub mod crypto;
pub mod error;
pub mod file_manager;
pub mod key_manager;

pub use error::DeviceError;
pub use file_manager::{
    exists_key_file, get_existing_key_pair_manager, init_key_pair_manager, reset_did_keys,
};
pub use key_manager::{KeyManager, PairKeyManager};
