use std::{fs, io, path::Path, str::FromStr};

use super::{crypto::get_random_bytes, error::DeviceError, key_manager::PairKeyManager};
use crate::{dto::Credential, routes::Mode};

const KEY_FILE_PATH: &str = "./keys.json";
const BASE_CLAIM_PRODUCTION_PATH: &str = "./base_claim_production.json";
const BASE_CLAIM_PRESENTATION_PATH: &str = "./base_claim_presentation.json";

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KeysFileStructure {
    pub payment_account_seed: String,
    pub did_auth_seed: String,
}

/// Save the key file to the specified path.
fn save_key_file(key_file: &KeysFileStructure) -> Result<(), io::Error> {
    let keys_file_json = serde_json::to_string_pretty(key_file)?;
    fs::write(KEY_FILE_PATH, keys_file_json)
}

pub fn get_existing_key_pair_manager() -> anyhow::Result<PairKeyManager> {
    let keys_file_json = fs::read_to_string(KEY_FILE_PATH)?;
    let keys_file: KeysFileStructure = serde_json::from_str(&keys_file_json)?;
    let payment_mnemonic = bip39::Mnemonic::from_str(&keys_file.payment_account_seed)?;
    let auth_mnemonic = bip39::Mnemonic::from_str(&keys_file.did_auth_seed)?;
    let manager = PairKeyManager::new(&payment_mnemonic.to_string(), &auth_mnemonic.to_string())?;
    Ok(manager)
}

pub fn exists_key_file() -> bool {
    Path::new(&KEY_FILE_PATH).exists()
}

/// Initialize keys and return a `PairKeyManager`.
pub fn init_key_pair_manager() -> anyhow::Result<PairKeyManager> {
    let key_file = generate_key_file_struct()?;
    save_key_file(&key_file)?;
    let manager = PairKeyManager::try_from(key_file)?;
    Ok(manager)
}

fn generate_key_file_struct() -> Result<KeysFileStructure, DeviceError> {
    let payment_random_seed = get_random_bytes(32)?;
    let auth_random_seed = get_random_bytes(32)?;
    let payment_mnemonic = bip39::Mnemonic::from_entropy(&payment_random_seed)?;
    let auth_mnemonic = bip39::Mnemonic::from_entropy(&auth_random_seed)?;

    Ok(KeysFileStructure {
        payment_account_seed: payment_mnemonic.to_string(),
        did_auth_seed: auth_mnemonic.to_string(),
    })
}

/// Reset keys and return a new `PairKeyManager`.
pub fn reset_did_keys() -> Result<PairKeyManager, DeviceError> {
    if Path::new(&KEY_FILE_PATH).exists() {
        // Generate a new authentication mnemonic
        let auth_random_seed = get_random_bytes(32)?;
        let auth_mnemonic = bip39::Mnemonic::from_entropy(&auth_random_seed)?;

        // Update key file with the new authentication mnemonic
        let mut keys_file: KeysFileStructure =
            serde_json::from_str(&fs::read_to_string(KEY_FILE_PATH)?)?;
        keys_file.did_auth_seed = auth_mnemonic.to_string();
        save_key_file(&keys_file)?;

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

fn get_claim_path(mode: Mode) -> &'static str {
    if mode == Mode::Presentation {
        BASE_CLAIM_PRESENTATION_PATH
    } else {
        BASE_CLAIM_PRODUCTION_PATH
    }
}

/// Reads the content in [BASE_CLAIM_PATH]
pub fn get_claim_content(mode: Mode) -> Result<Credential, DeviceError> {
    let base_claim_path = get_claim_path(mode);
    let base_claim = std::fs::read_to_string(base_claim_path)?;
    let claim: Credential = serde_json::from_str(&base_claim)?;
    Ok(claim)
}

/// saves the credential in [BASE_CLAIM_PATH]
pub fn save_claim_content(content: &Credential, mode: Mode) -> Result<(), DeviceError> {
    let base_claim_path = get_claim_path(mode);
    let string_content = serde_json::to_string(content)?;
    std::fs::write(base_claim_path, &string_content).map_err(DeviceError::from)
}
