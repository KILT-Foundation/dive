mod configuration;
mod device;
mod dto;
mod error;
mod http_client;
mod kilt;
mod routes;
mod utils;

#[cfg(test)]
mod test_utils;

use actix_cors::Cors;
use actix_files as fs;
use actix_session::{
    config::{CookieContentSecurity, PersistentSession},
    storage::CookieSessionStore,
    SessionMiddleware,
};
use actix_web::cookie::Key;
use actix_web::{cookie::time::Duration, middleware::Logger, web, App, HttpServer};
use clap::Parser;
use routes::{
    get_challenge_scope, get_claim_scope, get_credential_scope, get_did_scope, get_payment_scope,
};
use sodiumoxide::crypto::box_::SecretKey;
use std::sync::Arc;
use subxt::{ext::sp_core::sr25519::Pair, tx::PairSigner, utils::AccountId32, OnlineClient};
use tokio::sync::Mutex;
use utils::parse_configuration_to_app_state;

use crate::{
    configuration::Configuration,
    device::key_manager::PairKeyManager,
    kilt::{well_known_did_configuration::WellKnownDidConfig, KiltConfig},
    routes::get_well_known_did_config_scope,
};

#[derive(Clone)]
pub struct AppState {
    // key manager for handling the Did keys and payment account
    pub key_manager: Arc<Mutex<PairKeyManager>>,
    // api instance to interact with the blockchain.
    pub chain_client: Arc<OnlineClient<KiltConfig>>,
    pub auth_endpoint: String,
    pub attester_endpoint: String,
    pub auth_client_id: String,
    // jwt token used for login to the attester service. Created by OpenDid instance
    pub jwt_token: Arc<Mutex<String>>,
    // payment addr for paying tx
    pub payment_addr: String,
    // Did addr for the olibox
    pub did_addr: String,
    // Redirect url needed for OpenDid
    pub redirect_url: String,
    pub well_known_did_config: WellKnownDidConfig,
    // App name for creating credentials
    pub app_name: String,
    pub well_known_key_uri: String,
    // Public key
    pub session_encryption_public_key_uri: String,
    // Secret key for encryption. Needed for credential api
    pub secret_key: SecretKey,
    // key pair for creating credentials
    pub signer: Arc<PairSigner<KiltConfig, Pair>>,
    // Did for creating credentials
    pub did_attester: AccountId32,
}

pub async fn run(config: Configuration) -> anyhow::Result<()> {
    // if a thread receives a poisoned lock we panic the main thread.

    let port = config.port;
    let source_dir = config.front_end_path.clone();

    let app_state = parse_configuration_to_app_state(config).await?;

    utils::set_panic_hook();

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
                    .cookie_content_security(CookieContentSecurity::Private)
                    .cookie_http_only(true)
                    .cookie_secure(false)
                    .cookie_name("olibox".to_string())
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::seconds(60)),
                    )
                    .build(),
            )
            .wrap(Logger::default())
            .app_data(web::Data::new(app_state.clone()))
            //Did routes
            .service(get_did_scope())
            // claim routes
            .service(get_claim_scope())
            // payment routes
            .service(get_payment_scope())
            // Credential routes
            .service(get_credential_scope())
            // Well known did config
            .service(get_well_known_did_config_scope())
            //Challenge
            .service(get_challenge_scope())
            // Frontend
            .service(fs::Files::new("/", &source_dir).index_file("index.html"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;
    Ok(())
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Fetching env variables");
    let config = Configuration::parse();

    run(config).await
}
