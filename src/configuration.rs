use clap::Parser;
use serde::Deserialize;
use sodiumoxide::crypto::box_::SecretKey;
use subxt::{
    ext::sp_core::{sr25519, Pair},
    tx::PairSigner,
    utils::AccountId32,
};

use crate::kilt::{
    well_known_did_configuration::{create_well_known_did_config, WellKnownDidConfig},
    KiltConfig,
};

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
    pub well_known_origin: String,
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
    pub fn get_well_known_did_config(&self) -> anyhow::Result<WellKnownDidConfig> {
        create_well_known_did_config(
            &self.well_known_did,
            &self.well_known_key_uri,
            &self.well_known_origin,
            &self.well_known_seed,
        )
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

    #[cfg(test)]
    pub fn get_test_config() -> Self {
        Configuration {
            wss_address: "wss://peregrine.kilt.io:443/parachain-public-ws".to_string(),
            port: 7777,
            front_end_path: "does/not/matter".to_string(),
            attester_endpoint: "http://0.0.0.0:4444".to_string(),
            auth_endpoint: "http://0.0.0.0:5656".to_string(),
            auth_client_id: "default".to_string(),
            redirect_url: "http://0.0.0.0:3333".to_string(),
            well_known_did: "did:kilt:4qGqegcXWctkdLToCSFfBAUQE5V5SBdwivaB6miWgXS6C6Cf".to_string(),
            well_known_origin: "http://localhost:3333".to_string(),
            well_known_key_uri: "http://localhost:3333".to_string(),
            well_known_seed: "pave pattern upon invest humble squirrel cram flight wood travel already hint//did//assertion//0".to_string(),
            session_encryption_public_key_uri: "did:kilt:4qGqegcXWctkdLToCSFfBAUQE5V5SBdwivaB6miWgXS6C6Cf#0x4b77a1fb88c91ddd6e5ca84c821fa2c9c2f8fddb753c0f63afdda8c75c5e13af".to_string(),
            session_encryption_key_secret: "0x2f6d186c07aca73f746e4cfe3d2600446bb518c333c77afd4fa44f62aec281b8".to_string(),
            attestation_seed: "pave pattern upon invest humble squirrel cram flight wood travel already hint//did//assertion//0".to_string(),
            attestation_did_seed: "pave pattern upon invest humble squirrel cram flight wood travel already hint//did//0".to_string(),
        }
    }
}
