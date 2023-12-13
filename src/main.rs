mod configuration;
mod device;
mod dto;
mod error;
mod htttp_client;
mod kilt;
mod routes;

use actix_files as fs;
use actix_web::{middleware::Logger, web, App, HttpServer};
use clap::Parser;
use std::sync::{Arc, Mutex};
use subxt::{ext::sp_core::crypto::Ss58Codec, OnlineClient};

use crate::{
    configuration::Configuration,
    device::{
        exists_key_file, get_existing_key_pair_manager, init_key_pair_manager,
        key_manager::{KeyManager, PairKeyManager},
    },
    error::ServerError,
    kilt::{
        did_helper::{ADDRESS_FORMAT, DID_PREFIX},
        KiltConfig,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub key_manager: Arc<Mutex<PairKeyManager>>,
    pub kilt_api: Arc<Mutex<OnlineClient<KiltConfig>>>,
    pub auth_endpoint: String,
    pub attester_endpoint: String,
    pub auth_client_id: String,
    pub jwt_token: Arc<Mutex<String>>,
    pub payment_addr: String,
    pub did_addr: String,
}

pub async fn run(
    source_dir: String,
    wss_endpoint: String,
    port: u16,
    key_manager: PairKeyManager,
    auth_endpoint: String,
    attester_endpoint: String,
    auth_client_id: String,
) -> Result<(), ServerError> {
    let payment_account_id = key_manager.get_payment_account_signer().account_id();
    let did_auth_account_id = key_manager.get_did_auth_signer().account_id();

    let payment_addr = payment_account_id.to_ss58check_with_version(ADDRESS_FORMAT.into());
    let did_addr = did_auth_account_id.to_ss58check_with_version(ADDRESS_FORMAT.into());

    log::info!("payment_account_id: {}", payment_addr);
    log::info!("DID: {}{}", DID_PREFIX, did_addr);

    let api = kilt::connect(&wss_endpoint).await?;

    log::info!("Connected to: {}", wss_endpoint);

    let app_state = AppState {
        key_manager: Arc::new(Mutex::new(key_manager)),
        kilt_api: Arc::new(Mutex::new(api)),
        jwt_token: Arc::new(Mutex::new(String::new())),
        attester_endpoint,
        auth_client_id,
        auth_endpoint,
        payment_addr,
        did_addr,
    };

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(app_state.clone()))
            //Did routes
            .service(
                web::scope("/api/v1/did")
                    .route("", web::delete().to(crate::routes::reset))
                    .route("", web::get().to(crate::routes::get_did))
                    .route("", web::post().to(crate::routes::register_device_did)),
            )
            // claim routes
            .service(
                web::scope("/api/v1/claim")
                    .route("", web::post().to(crate::routes::post_base_claim))
                    .route("", web::get().to(crate::routes::get_base_claim)),
            )
            // payment routes
            .service(
                web::scope("/api/v1/payment")
                    .route(
                        "",
                        web::get().to(crate::routes::get_payment_account_address),
                    )
                    .route("", web::post().to(crate::routes::submit_extrinsic)),
            )
            // Credential routes
            .service(
                web::scope("/api/v1/credential")
                    .route("", web::get().to(crate::routes::get_credential)),
            )
            .service(fs::Files::new("/", &source_dir).index_file("index.html"))
    })
    .bind(("0.0.0.0", port))
    .map_err(|e| ServerError::IO(e))?
    .run()
    .await
    .map_err(|e| ServerError::IO(e))
}

#[actix_web::main]
async fn main() -> Result<(), ServerError> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Fetching env variables");

    let config = Configuration::parse();

    let source_dir = config.front_end_path;
    let wss_endpoint = config.wss_address;
    let port = config.port;
    let auth_endpoint = config.auth_endpoint;
    let attester_endpoint = config.attester_endpoint;
    let auth_client_id = config.auth_client_id;

    let key_manager = {
        if exists_key_file() {
            get_existing_key_pair_manager()?
        } else {
            init_key_pair_manager()?
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
    )
    .await
}
