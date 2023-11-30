use std::sync::{Arc, Mutex};

use actix_files as fs;
use actix_web::{web, App, HttpServer};
use subxt::{ext::sp_core::crypto::Ss58Codec, OnlineClient};

use crate::{
    device::key_manager::{KeyManager, PairKeyManager},
    kilt::{self, KiltConfig},
    server::{error::ServerError, routes::get_payment_account_address},
};

use super::routes::submit_extrinsic;

#[derive(Clone)]
pub struct AppState {
    pub key_manager: Arc<Mutex<PairKeyManager>>,
    pub kilt_api: Arc<Mutex<OnlineClient<KiltConfig>>>,
    pub credential_id: String,
    pub auth_endpoint: String,
    pub attester_endpoint: String,
    pub auth_client_id: String,
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

    log::info!(
        "payment_account_id: {}",
        payment_account_id.to_ss58check_with_version(38u16.into())
    );
    log::info!(
        "DID: did:kilt:{}",
        did_auth_account_id.to_ss58check_with_version(38u16.into())
    );

    let api = kilt::connect(&wss_endpoint).await?;

    log::info!("Connected to: {}", wss_endpoint);

    let app_state = AppState {
        key_manager: Arc::new(Mutex::new(key_manager)),
        kilt_api: Arc::new(Mutex::new(api)),
        credential_id: "".to_string(),
        attester_endpoint,
        auth_client_id,
        auth_endpoint,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            //Did routes
            .service(
                web::scope("/api/v1/did")
                    .route("", web::delete().to(crate::server::routes::reset))
                    .route("", web::get().to(crate::server::routes::get_did))
                    .route(
                        "",
                        web::post().to(crate::server::routes::register_device_did),
                    ),
            )
            // claim routes
            .service(
                web::scope("/api/v1/claim")
                    .route("", web::post().to(crate::server::routes::post_base_claim))
                    .route("", web::get().to(crate::server::routes::get_base_claim)),
            )
            // payment routes
            .service(
                web::scope("/api/v1/payment")
                    .route("", web::get().to(get_payment_account_address))
                    .route("", web::post().to(submit_extrinsic)),
            )
            // Credential routes
            .service(
                web::scope("/api/v1/credential")
                    .route("", web::get().to(crate::server::routes::get_credential)),
            )
            .service(fs::Files::new("/", &source_dir).index_file("index.html"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
    .map_err(ServerError::from)
}
