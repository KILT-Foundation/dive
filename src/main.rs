mod configuration;
mod device;
mod dto;
mod error;
mod http_client;
mod kilt;
mod routes;
mod utils;

use actix_cors::Cors;
use actix_files as fs;
use actix_session::{
    config::{CookieContentSecurity, PersistentSession},
    storage::CookieSessionStore,
    SessionMiddleware,
};
use actix_web::{
    cookie::{time::Duration, Key},
    middleware::Logger,
    web, App, HttpServer,
};
use anyhow::Context;
use clap::Parser;
use routes::{
    get_challenge_scope, get_claim_scope, get_credential_scope, get_did_scope, get_payment_scope,
    get_use_case_scope,
};
use sodiumoxide::crypto::box_::SecretKey;
use std::sync::Arc;
use subxt::{
    ext::sp_core::{crypto::Ss58Codec, sr25519::Pair},
    tx::PairSigner,
    utils::AccountId32,
    OnlineClient,
};
use tokio::sync::Mutex;

use crate::{
    configuration::Configuration,
    device::{
        exists_key_file, get_existing_key_pair_manager, init_key_pair_manager,
        key_manager::{KeyManager, PairKeyManager},
    },
    kilt::{
        did_helper::{ADDRESS_FORMAT, DID_PREFIX},
        well_known_did_configuration::WellKnownDidConfigData,
        KiltConfig,
    },
    routes::get_well_known_did_config_scope,
};

const SERVICE_ENDPOINT_TYPE: &'static str = "KiltPublishedCredentialCollectionV1Type";
const USE_CASE_SERVICE_ENDPOINT_ID: &'static str = "dive";

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
    // App name for creating credentials
    pub app_name: String,
    // Public key
    pub session_encryption_public_key_uri: String,
    // Secret key for encryption. Needed for credential api
    pub secret_key: SecretKey,
    // key pair for creating credentials
    pub signer: Arc<PairSigner<KiltConfig, Pair>>,
    // Did for creating credentials
    pub did_attester: AccountId32,
    /// Type for service Endpoint
    pub kilt_service_endpoint_type: String,
    ///Service Endpoint ID for use case participation
    pub use_case_service_endpoint_id: String,
    /// Used data to create the well known did config
    pub well_known_did_config_data: Arc<Mutex<WellKnownDidConfigData>>,
}

pub async fn run(
    source_dir: String,
    wss_endpoint: String,
    port: u16,
    key_manager: PairKeyManager,
    auth_endpoint: String,
    attester_endpoint: String,
    auth_client_id: String,
    redirect_url: String,
    session_encryption_public_key_uri: String,
    secret_key: SecretKey,
    signer: PairSigner<KiltConfig, Pair>,
    did_attester: AccountId32,
    well_known_did_config_data: WellKnownDidConfigData,
) -> anyhow::Result<()> {
    let payment_signer = key_manager.get_payment_account_signer();
    let payment_account_id = payment_signer.account_id();
    let did_auth_signer = key_manager.clone().get_did_auth_signer();
    let did_account_id = did_auth_signer.account_id();

    let payment_addr = payment_account_id.to_ss58check_with_version(ADDRESS_FORMAT.into());
    let did_addr = did_account_id.to_ss58check_with_version(ADDRESS_FORMAT.into());

    log::info!("payment_account_id: {}", payment_addr);
    log::info!("Olibox DID: {}{}", DID_PREFIX, did_addr);

    let api = OnlineClient::<KiltConfig>::from_url(&wss_endpoint)
        .await
        .context("Creating the onlineclient should not fail.")?;

    log::info!("Connected to: {}", wss_endpoint);

    log::info!("Source dir: {}", source_dir);

    let app_state = AppState {
        key_manager: Arc::new(Mutex::new(key_manager)),
        chain_client: Arc::new(api),
        jwt_token: Arc::new(Mutex::new(String::new())),
        signer: Arc::new(signer),
        well_known_did_config_data: Arc::new(Mutex::new(well_known_did_config_data)),
        app_name: "Olibox".to_string(),
        attester_endpoint,
        auth_client_id,
        auth_endpoint,
        payment_addr,
        did_addr,
        redirect_url,
        session_encryption_public_key_uri,
        secret_key,
        did_attester,
        kilt_service_endpoint_type: String::from(SERVICE_ENDPOINT_TYPE),
        use_case_service_endpoint_id: String::from(USE_CASE_SERVICE_ENDPOINT_ID),
    };

    // if a thread receives a poisoned lock we panic the main thread.
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
            //Use case routes
            .service(get_use_case_scope())
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

    let well_known_did_config_data = config.get_well_known_did_config_data();

    let secret_key = config.get_secret_key()?;
    let did_attester = config.get_did()?;

    let signer = config.get_credential_signer()?;
    let source_dir = config.front_end_path;
    let wss_endpoint = config.wss_address;
    let port = config.port;
    let auth_endpoint = config.auth_endpoint;
    let attester_endpoint = config.attester_endpoint;
    let auth_client_id = config.auth_client_id;
    let redirect_url = config.redirect_url;
    let session_encryption_public_key_uri = config.session_encryption_public_key_uri;

    let key_manager = {
        if exists_key_file() {
            get_existing_key_pair_manager()
                .context("Fetching existing key pairs from file system should not fail.")?
        } else {
            init_key_pair_manager().context("Init new key pair should not fail.")?
        }
    };

    log::info!("Staring Server on port: {}", port);

    run(
        source_dir,
        wss_endpoint,
        port,
        key_manager,
        auth_endpoint,
        attester_endpoint,
        auth_client_id,
        redirect_url,
        session_encryption_public_key_uri,
        secret_key,
        signer,
        did_attester,
        well_known_did_config_data,
    )
    .await
}
