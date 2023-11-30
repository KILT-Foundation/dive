use std::{fs, io, path::Path, str::FromStr};

use super::{crypto::get_random_bytes, error::DeviceError, key_manager::PairKeyManager};

const DIR_PATH: String = "/etc/kilt".to_string();
const KEY_FILE_PATH: String = "/etc/kilt/keys.json".to_string();

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct KeysFileStructure {
    payment_account_seed: String,
    did_auth_seed: String,
}

/// Save the key file to the specified path.
fn save_file(key_file: &KeysFileStructure) -> Result<(), io::Error> {
    let keys_file_json = serde_json::to_string_pretty(key_file)?;
    fs::create_dir_all(DIR_PATH)?;
    fs::write(KEY_FILE_PATH, keys_file_json)
}

/// Initialize keys and return a `PairKeyManager`.
pub fn init_keys() -> Result<PairKeyManager, DeviceError> {
    if !Path::new(&KEY_FILE_PATH).exists() {
        // Generate random seeds and mnemonics
        let payment_random_seed = get_random_bytes(32)?;
        let auth_random_seed = get_random_bytes(32)?;
        let payment_mnemonic = bip39::Mnemonic::from_entropy(&payment_random_seed)?;
        let auth_mnemonic = bip39::Mnemonic::from_entropy(&auth_random_seed)?;

        // Create key file structure
        let key_file = KeysFileStructure {
            payment_account_seed: payment_mnemonic.to_string(),
            did_auth_seed: auth_mnemonic.to_string(),
        };

        // Save key file and initialize PairKeyManager
        save_file(&key_file)?;
        let manager =
            PairKeyManager::new(&payment_mnemonic.to_string(), &auth_mnemonic.to_string())?;
        Ok(manager)
    } else {
        // Read existing key file and initialize PairKeyManager
        let keys_file_json = fs::read_to_string(KEY_FILE_PATH)?;
        let keys_file: KeysFileStructure = serde_json::from_str(&keys_file_json)?;
        let payment_mnemonic = bip39::Mnemonic::from_str(&keys_file.payment_account_seed)?;
        let auth_mnemonic = bip39::Mnemonic::from_str(&keys_file.did_auth_seed)?;
        let manager =
            PairKeyManager::new(&payment_mnemonic.to_string(), &auth_mnemonic.to_string())?;
        Ok(manager)
    }
}

/// Reset keys and return a new `PairKeyManager`.
pub fn reset_keys() -> Result<PairKeyManager, DeviceError> {
    if Path::new(&KEY_FILE_PATH).exists() {
        // Generate a new authentication mnemonic
        let auth_random_seed = get_random_bytes(32)?;
        let auth_mnemonic = bip39::Mnemonic::from_entropy(&auth_random_seed)?;

        // Update key file with the new authentication mnemonic
        let mut keys_file: KeysFileStructure =
            serde_json::from_str(&fs::read_to_string(KEY_FILE_PATH)?)?;
        keys_file.did_auth_seed = auth_mnemonic.to_string();
        save_file(&keys_file)?;

        // Initialize and return the new PairKeyManager
        let payment_mnemonic = bip39::Mnemonic::from_str(&keys_file.payment_account_seed)?;
        let auth_mnemonic = bip39::Mnemonic::from_str(&keys_file.did_auth_seed)?;
        let manager =
            PairKeyManager::new(&payment_mnemonic.to_string(), &auth_mnemonic.to_string())?;
        Ok(manager)
    } else {
        // Return an error if the key file is not found
        Err(DeviceError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "Key file not found",
        )))
    }
}
