use std::sync::Arc;

use actix_rt::System;
use anyhow::Context;
use serde::de::Error;
use serde::Deserialize;
use serde::{Deserializer, Serializer};
use subxt::ext::sp_core::crypto::Ss58Codec;
use subxt::OnlineClient;
use tokio::sync::Mutex;

use crate::configuration::Configuration;
use crate::device::key_manager::KeyManager;
use crate::device::{exists_key_file, get_existing_key_pair_manager, init_key_pair_manager};
use crate::kilt::did_helper::{ADDRESS_FORMAT, DID_PREFIX};
use crate::kilt::KiltConfig;
use crate::AppState;

/// if a thread panics we kill the whole process gracefully
pub fn set_panic_hook() {
    let orig_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // shut down the other threads gracefully
        System::current().stop();

        // continue to propagate the panic.
        orig_hook(panic_info);
    }));
}

pub async fn parse_configuration_to_app_state(config: Configuration) -> anyhow::Result<AppState> {
    let well_known_did_config = config
        .clone()
        .get_well_known_did_config()
        .context("Creating Well known did config failed")?;

    let secret_key = config.get_secret_key()?;
    let did_attester = config.get_did()?;

    let signer = config.get_credential_signer()?;
    let source_dir = config.front_end_path;
    let wss_endpoint = config.wss_address;
    let auth_endpoint = config.auth_endpoint;
    let attester_endpoint = config.attester_endpoint;
    let auth_client_id = config.auth_client_id;
    let redirect_url = config.redirect_url;
    let well_known_key_uri = config.well_known_key_uri;
    let session_encryption_public_key_uri = config.session_encryption_public_key_uri;

    let key_manager = {
        if exists_key_file() {
            get_existing_key_pair_manager()
                .context("Fetching existing key pairs from file system should not fail.")?
        } else {
            init_key_pair_manager().context("Init new key pair should not fail.")?
        }
    };

    let payment_signer = key_manager.get_payment_account_signer();
    let payment_account_id = payment_signer.account_id();
    let did_auth_signer = key_manager.clone().get_did_auth_signer();
    let did_account_id = did_auth_signer.account_id();

    let payment_addr = payment_account_id.to_ss58check_with_version(ADDRESS_FORMAT.into());
    let did_addr = did_account_id.to_ss58check_with_version(ADDRESS_FORMAT.into());

    let api = OnlineClient::<KiltConfig>::from_url(&wss_endpoint)
        .await
        .context("Creating the onlineclient should not fail.")?;

    log::info!("Connected to: {}", wss_endpoint);
    log::info!("Source dir: {}", source_dir);
    log::info!("payment_account_id: {}", payment_addr);
    log::info!("Olibox DID: {}{}", DID_PREFIX, did_addr);

    Ok(AppState {
        key_manager: Arc::new(Mutex::new(key_manager)),
        chain_client: Arc::new(api),
        jwt_token: Arc::new(Mutex::new(String::new())),
        signer: Arc::new(signer),
        app_name: "Olibox".to_string(),
        attester_endpoint,
        auth_client_id,
        auth_endpoint,
        payment_addr,
        did_addr,
        redirect_url,
        well_known_did_config,
        well_known_key_uri,
        session_encryption_public_key_uri,
        secret_key,
        did_attester,
    })
}

/// Serialize and deserialize `0x` prefixed hex strings to and from Vec<u8>.
pub mod prefixed_hex {

    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        hex::decode(s.trim_start_matches("0x")).map_err(Error::custom)
    }

    pub fn serialize<S>(value: &Vec<u8>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = hex::encode(value);
        s.serialize_str(&format!("0x{}", hex))
    }
}

/// Serialize and deserialize `0x` prefixed hex strings to and from Vec<u8>.
pub mod hex_nonce {
    use sodiumoxide::crypto::box_::Nonce;

    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Nonce, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let bytes = hex::decode(s.trim_start_matches("0x")).map_err(Error::custom)?;
        Nonce::from_slice(&bytes).ok_or_else(|| Error::custom("invalid nonce length"))
    }

    pub fn serialize<S>(value: &Nonce, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = hex::encode(value);
        s.serialize_str(&format!("0x{}", hex))
    }
}
