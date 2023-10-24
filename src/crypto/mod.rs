use std::str::FromStr;

use self::manager::PairKeyManager;

#[cfg(feature = "hsm6")]
pub mod device;

#[cfg(not(feature = "hsm6"))]
use rand::Rng;

pub mod manager;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct KeysFileStructure {
    payment_account_seed: String,
    pub did_auth_seed: String,
}

#[cfg(feature = "hsm6")]
fn get_random_bytes(num_bytes: i32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let hsm6 = device::ZkCtx::new()?;
    let random_bytes = hsm6.get_random_bytes(num_bytes)?;
    Ok(random_bytes)
}

#[cfg(not(feature = "hsm6"))]
fn get_random_bytes(_num_bytes: i32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut random_bytes = vec![0u8; 32];
    rand::thread_rng().fill(&mut random_bytes[..]);
    Ok(random_bytes)
}

pub fn init_keys() -> Result<PairKeyManager, Box<dyn std::error::Error>> {
    // first check if keys are already initialized by checking /etc/kilt/keys.json file
    let keys_file_path = "/etc/kilt/keys.json";
    // check if file exists
    if !std::path::Path::new(keys_file_path).exists() {
        let payment_random_seed = get_random_bytes(32)?;
        let auth_random_seed = get_random_bytes(32)?;
        let payment_mnemonic = bip39::Mnemonic::from_entropy(&payment_random_seed)?;
        let auth_mnemonic = bip39::Mnemonic::from_entropy(&auth_random_seed)?;
        let keys_file = KeysFileStructure {
            payment_account_seed: payment_mnemonic.to_string(),
            did_auth_seed: auth_mnemonic.to_string(),
        };
        let keys_file_json = serde_json::to_string_pretty(&keys_file)?;
        std::fs::create_dir_all("/etc/kilt")?;
        std::fs::write(keys_file_path, keys_file_json)?;
        let manager =
            PairKeyManager::new(&payment_mnemonic.to_string(), &auth_mnemonic.to_string())?;
        Ok(manager)
    } else {
        let keys_file_json = std::fs::read_to_string(keys_file_path)?;
        let keys_file: KeysFileStructure = serde_json::from_str(&keys_file_json)?;
        let payment_mnemonic = bip39::Mnemonic::from_str(&keys_file.payment_account_seed)?;
        let auth_mnemonic = bip39::Mnemonic::from_str(&keys_file.did_auth_seed)?;
        let manager =
            PairKeyManager::new(&payment_mnemonic.to_string(), &auth_mnemonic.to_string())?;
        Ok(manager)
    }
}

pub fn reset_did_keys() -> Result<(), Box<dyn std::error::Error>> {
    let keys_file_path = "/etc/kilt/keys.json";

    if std::path::Path::new(keys_file_path).exists() {
        // init new keys for did
        let auth_random_seed = get_random_bytes(32)?;
        let auth_mnemonic = bip39::Mnemonic::from_entropy(&auth_random_seed)?;

        //
        let keys_file_json = std::fs::read_to_string(keys_file_path)?;
        let mut keys_file: KeysFileStructure = serde_json::from_str(&keys_file_json)?;
        keys_file.did_auth_seed = auth_mnemonic.to_string();

        let keys_fil_json = serde_json::to_string_pretty(&keys_file)?;

        std::fs::write(keys_file_path, keys_fil_json)?;
    }
    Ok(())
}
