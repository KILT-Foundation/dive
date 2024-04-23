use std::{fs, io, path::Path, str::FromStr};
use subxt::ext::sp_core::crypto::Ss58Codec;

use crate::{
    device::{
        crypto::get_random_bytes,
        error::DeviceError,
        key_manager::{KeyManager, PairKeyManager},
    },
    dto::Credential,
    kilt::did_helper::{ADDRESS_FORMAT, DID_PREFIX},
};

const KEY_FILE_PATH: &str = "./keys.json";
pub const BASE_CLAIM_PATH: &str = "./base_claim.json";

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KeysFileStructure {
    pub payment_account_seed: String,
    pub did_auth_seed: String,
    pub did: String,
}

/// Save the key file to [KEY_FILE_PATH].
fn save_key_file(key_file: &KeysFileStructure) -> Result<(), io::Error> {
    let keys_file_json = serde_json::to_string_pretty(key_file)?;
    fs::write(KEY_FILE_PATH, keys_file_json)
}

/// Reads the content of [KEY_FILE_PATH]
pub fn get_existing_key_pair_manager() -> anyhow::Result<PairKeyManager> {
    let keys_file_json = fs::read_to_string(KEY_FILE_PATH)?;
    let keys_file: KeysFileStructure = serde_json::from_str(&keys_file_json)?;
    let payment_mnemonic = bip39::Mnemonic::from_str(&keys_file.payment_account_seed)?;
    let auth_mnemonic = bip39::Mnemonic::from_str(&keys_file.did_auth_seed)?;
    let manager = PairKeyManager::new(&payment_mnemonic.to_string(), &auth_mnemonic.to_string())?;
    Ok(manager)
}

/// checks if [KEY_FILE_PATH] exists
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

/// generates key file struct containing: Did keys, Payment keys and DID identifier
fn generate_key_file_struct() -> Result<KeysFileStructure, DeviceError> {
    let payment_random_seed = get_random_bytes(32)?;
    let auth_random_seed = get_random_bytes(32)?;
    let payment_mnemonic = bip39::Mnemonic::from_entropy(&payment_random_seed)?;
    let auth_mnemonic = bip39::Mnemonic::from_entropy(&auth_random_seed)?;

    let manager = PairKeyManager::new(&payment_mnemonic.to_string(), &auth_mnemonic.to_string())?;
    let raw_did = manager
        .get_did_auth_signer()
        .account_id()
        .to_ss58check_with_version(ADDRESS_FORMAT.into());

    let did = format!("{}{}", DID_PREFIX, raw_did);

    Ok(KeysFileStructure {
        payment_account_seed: payment_mnemonic.to_string(),
        did_auth_seed: auth_mnemonic.to_string(),
        did,
    })
}

/// Resets DID keys and DID identifier and returns a new `PairKeyManager`.
pub fn reset_did_keys() -> Result<PairKeyManager, DeviceError> {
    if Path::new(&KEY_FILE_PATH).exists() {
        // Generate a new authentication mnemonic
        let auth_random_seed = get_random_bytes(32)?;
        let auth_mnemonic = bip39::Mnemonic::from_entropy(&auth_random_seed)?;

        // Update key file with the new authentication mnemonic
        let mut keys_file: KeysFileStructure =
            serde_json::from_str(&fs::read_to_string(KEY_FILE_PATH)?)?;
        keys_file.did_auth_seed = auth_mnemonic.to_string();

        let manager =
            PairKeyManager::new(&keys_file.payment_account_seed, &keys_file.did_auth_seed)?;

        let raw_did = manager
            .get_did_auth_signer()
            .account_id()
            .to_ss58check_with_version(ADDRESS_FORMAT.into());

        let did = format!("{}{}", DID_PREFIX, raw_did);

        keys_file.did = did;
        save_key_file(&keys_file)?;

        Ok(manager)
    } else {
        // Return an error if the key file is not found
        Err(DeviceError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "Key file not found",
        )))
    }
}

/// Reads the content in [BASE_CLAIM_PATH]
pub fn get_claim_content() -> Result<Credential, DeviceError> {
    let base_claim = std::fs::read_to_string(BASE_CLAIM_PATH)?;
    let claim: Credential = serde_json::from_str(&base_claim)?;
    Ok(claim)
}

/// saves the credential in [BASE_CLAIM_PATH]
pub fn save_claim_content(content: &Credential) -> Result<(), DeviceError> {
    let string_content = serde_json::to_string(content)?;
    std::fs::write(BASE_CLAIM_PATH, &string_content).map_err(DeviceError::from)
}
