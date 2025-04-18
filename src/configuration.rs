use clap::Parser;
use serde::Deserialize;
use sodiumoxide::crypto::box_::SecretKey;
use subxt::{
    ext::sp_core::{sr25519, Pair},
    tx::PairSigner,
    utils::AccountId32,
};

use crate::kilt::{well_known_did_configuration::WellKnownDidConfigData, KiltConfig};

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
    pub well_known_did: String,
    #[clap(env)]
    pub well_known_key_uri: String,
    #[clap(env)]
    pub well_known_seed: String,
    #[clap(env)]
    pub session_encryption_public_key_uri: String,
    #[clap(env)]
    session_encryption_key_secret: String,
    #[clap(env)]
    attestation_seed: String,
    #[clap(env)]
    pub attestation_did_seed: String,
}

impl Configuration {
    pub fn get_well_known_did_config_data(&self) -> WellKnownDidConfigData {
        WellKnownDidConfigData {
            did: self.well_known_did.clone(),
            key_uri: self.well_known_key_uri.clone(),
            origin: String::new(),
            seed: self.well_known_seed.clone(),
        }
    }

    pub fn get_secret_key(&self) -> anyhow::Result<SecretKey> {
        let raw_key = hex::decode(self.session_encryption_key_secret.trim_start_matches("0x"))?;
        SecretKey::from_slice(&raw_key).ok_or(anyhow::anyhow!("Generating secret key failed"))
    }

    pub fn get_credential_signer(&self) -> anyhow::Result<PairSigner<KiltConfig, sr25519::Pair>> {
        let pair = sr25519::Pair::from_string_with_seed(&self.attestation_seed, None)?.0;
        Ok(PairSigner::new(pair))
    }
    pub fn get_did(&self) -> anyhow::Result<AccountId32> {
        let pair = sr25519::Pair::from_string_with_seed(&self.attestation_did_seed, None)?.0;
        Ok(pair.public().into())
    }
}
