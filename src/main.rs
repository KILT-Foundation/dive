mod configuration;
mod device;
mod dto;
mod error;
mod http_client;
mod kilt;
mod routes;
mod utils;

use actix_files as fs;
use actix_web::{middleware::Logger, web, App, HttpServer};
use anyhow::Context;
use clap::Parser;
use std::sync::{Arc, Mutex};
use subxt::{ext::sp_core::crypto::Ss58Codec, OnlineClient};

use crate::{
    configuration::Configuration,
    device::{
        exists_key_file, get_existing_key_pair_manager, init_key_pair_manager,
        key_manager::{KeyManager, PairKeyManager},
    },
    kilt::{
        did_helper::{ADDRESS_FORMAT, DID_PREFIX},
        KiltConfig,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub key_manager: Arc<Mutex<PairKeyManager>>,
    pub chain_client: Arc<OnlineClient<KiltConfig>>,
    pub auth_endpoint: String,
    pub attester_endpoint: String,
    pub auth_client_id: String,
    pub jwt_token: Arc<Mutex<String>>,
    pub payment_addr: String,
    pub did_addr: String,
    pub redirect_url: String,
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
) -> anyhow::Result<()> {
    let payment_account_id = key_manager.get_payment_account_signer().account_id();
    let did_auth_account_id = key_manager.get_did_auth_signer().account_id();

    let payment_addr = payment_account_id.to_ss58check_with_version(ADDRESS_FORMAT.into());
    let did_addr = did_auth_account_id.to_ss58check_with_version(ADDRESS_FORMAT.into());

    log::info!("payment_account_id: {}", payment_addr);
    log::info!("DID: {}{}", DID_PREFIX, did_addr);

    let api = OnlineClient::<KiltConfig>::from_url(&wss_endpoint)
        .await
        .context("Creating the onlineclient should not fail.")?;

    log::info!("Connected to: {}", wss_endpoint);

    let app_state = AppState {
        key_manager: Arc::new(Mutex::new(key_manager)),
        chain_client: Arc::new(api),
        jwt_token: Arc::new(Mutex::new(String::new())),
        attester_endpoint,
        auth_client_id,
        auth_endpoint,
        payment_addr,
        did_addr,
        redirect_url,
    };

    // if a thread receives a poisond lock we panic the main thread.
    utils::set_panic_hook();

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

    let source_dir = config.front_end_path;
    let wss_endpoint = config.wss_address;
    let port = config.port;
    let auth_endpoint = config.auth_endpoint;
    let attester_endpoint = config.attester_endpoint;
    let auth_client_id = config.auth_client_id;
    let redirect_url = config.redirect_url;

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
    )
    .await
}
