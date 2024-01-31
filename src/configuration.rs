use clap::Parser;
use serde::Deserialize;
use sodiumoxide::crypto::box_::SecretKey;

use crate::kilt::well_known_did_configuration::{create_well_known_did_config, WellKnownDidConfig};

#[derive(Deserialize, Debug, Clone, Parser)]
pub struct Configuration {
    #[clap(env)]
    pub wss_address: String,
    #[clap(env)]
    pub port: u16,
    #[clap(env)]
    pub front_end_path: String,
    #[clap(env)]
    pub attester_endpoint: String,
    #[clap(env)]
    pub auth_endpoint: String,
    #[clap(env)]
    pub auth_client_id: String,
    #[clap(env)]
    pub redirect_url: String,
    #[clap(env)]
    pub did: String,
    #[clap(env)]
    pub key_uri: String,
    #[clap(env)]
    pub origin: String,
    #[clap(env)]
    pub seed: String,
    #[clap(env)]
    pub encryption_key_uri: String,
    #[clap(env)]
    secret_encryption_key: String,
}

impl Configuration {
    pub fn get_well_known_did_config(&self) -> anyhow::Result<WellKnownDidConfig> {
        create_well_known_did_config(&self.did, &self.key_uri, &self.origin, &self.seed)
    }

    pub fn get_secret_key(&self) -> anyhow::Result<SecretKey> {
        let raw_key = hex::decode(self.secret_encryption_key.trim_start_matches("0x"))?;
        SecretKey::from_slice(&raw_key).ok_or(anyhow::anyhow!("Generating secret key failed"))
    }
}
