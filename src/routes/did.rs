use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use subxt::ext::sp_core::crypto::Ss58Codec;

use crate::{
    device::{file_manager::BASE_CLAIM_PATH, key_manager::KeyManager},
    dto::{DidAddress, TxResponse},
    error::ServerError,
    kilt::{
        connect,
        did_helper::{query_did_doc, ADDRESS_FORMAT, DID_PREFIX},
        tx::create_did,
    },
    AppState,
};

#[post("")]
async fn register_device_did(
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ServerError> {
    let keys = app_state.key_manager.lock().await;
    let did_auth_signer = &keys.get_did_auth_signer();
    let submitter_signer = &keys.get_payment_account_signer();
    let chain_client = connect(&app_state.wss_endpoint).await?;
    let extrinsic_hash = create_did(did_auth_signer, submitter_signer, &chain_client).await?;

    let tx = format!("0x{}", hex::encode(extrinsic_hash));
    log::info!("Tx hash: {}", tx);

    let did = did_auth_signer
        .account_id()
        .to_ss58check_with_version(ADDRESS_FORMAT.into());

    let formatted_did = format!("{}{}", DID_PREFIX, did);

    Ok(HttpResponse::Ok().json(TxResponse {
        tx,
        did: formatted_did,
    }))
}

#[get("")]
async fn get_did(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let keys = app_state.key_manager.lock().await;
    let did_auth_signer = &keys.get_did_auth_signer();
    let chain_client = connect(&app_state.wss_endpoint).await?;

    let did = did_auth_signer
        .account_id()
        .to_ss58check_with_version(ADDRESS_FORMAT.into());

    query_did_doc(&did, &chain_client).await?;
    Ok(HttpResponse::Ok().json(DidAddress {
        did: format!("{}{}", DID_PREFIX, did),
    }))
}

#[delete("")]
async fn reset(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let new_key_manager = crate::device::reset_did_keys()?;

    log::info!(
        "new Did: {:?}",
        new_key_manager.get_did_auth_signer().account_id()
    );

    let mut key_manager = app_state.key_manager.lock().await;
    *key_manager = new_key_manager;

    let _ = tokio::fs::remove_file(BASE_CLAIM_PATH).await;

    Ok(HttpResponse::Ok())
}

pub fn get_did_scope() -> Scope {
    web::scope("/api/v1/did")
        .service(reset)
        .service(get_did)
        .service(register_device_did)
}
