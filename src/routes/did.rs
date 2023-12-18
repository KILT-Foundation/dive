use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use std::io::ErrorKind;

use crate::{
    device::key_manager::KeyManager,
    dto::{DidAddress, TxResponse},
    error::ServerError,
    kilt::did_helper::{create_did, query_did_doc, DID_PREFIX},
    AppState,
};

#[post("")]
async fn register_device_did(
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ServerError> {
    let keys = app_state.key_manager.lock().await;
    let did_auth_signer = &keys.get_did_auth_signer();
    let submitter_signer = &keys.get_payment_account_signer();
    let chain_client = &app_state.chain_client;
    let extrinsic_hash = create_did(did_auth_signer, submitter_signer, chain_client).await?;
    let tx = format!("0x{}", hex::encode(extrinsic_hash));
    log::info!("Tx hash: {}", tx);
    Ok(HttpResponse::Ok().json(TxResponse { tx }))
}

#[get("")]
async fn get_did(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let addr = &app_state.did_addr;
    let chain_client = &app_state.chain_client;
    query_did_doc(&addr, chain_client).await?;
    Ok(HttpResponse::Ok().json(DidAddress {
        did: format!("{}{}", DID_PREFIX, addr),
    }))
}

#[delete("")]
async fn reset(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let new_key_manager = crate::device::reset_did_keys()?;

    log::info!(
        "new Did: {:?}",
        new_key_manager.get_did_auth_signer().account_id()
    );

    let remove_file_result = tokio::fs::remove_file("base_claim.json").await;

    if remove_file_result.is_err() {
        let err = remove_file_result.unwrap_err();

        if err.kind() == ErrorKind::NotFound {
            return Ok(HttpResponse::NotFound());
        }

        let device_err = err.into();
        return Err(ServerError::Device(device_err));
    }

    let mut key_manager = app_state.key_manager.lock().await;
    *key_manager = new_key_manager;
    Ok(HttpResponse::Ok())
}

pub fn get_did_scope() -> Scope {
    web::scope("/api/v1/did")
        .service(reset)
        .service(get_did)
        .service(register_device_did)
}
