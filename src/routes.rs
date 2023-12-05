use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use subxt::ext::sp_core::crypto::Ss58Codec;

use crate::{
    device::{
        file_manager::{get_claim_content, save_claim_content},
        key_manager::KeyManager,
    },
    dto::Credential,
    error::ServerError,
    htttp_client::{
        check_jwt_health, get_credentials_from_attester, login_to_open_did, post_claim_to_attester,
    },
    kilt::{
        did_helper::{create_did, get_did_doc},
        error::{FormatError, TxError},
        tx::{submit_call, BoxSigner, WaitFor},
    },
    AppState,
};

pub async fn get_payment_account_address(
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ServerError> {
    let key_manager = app_state.key_manager.lock()?;
    let payment_account_id = key_manager.get_payment_account_signer().account_id();
    let addr = payment_account_id.to_ss58check_with_version(38u16.into());
    Ok(HttpResponse::Ok().json(json!({ "address": addr })))
}

pub async fn get_did(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let key_manager = app_state.key_manager.lock()?;
    let did_auth_account_id = key_manager.get_did_auth_signer().account_id();
    let addr = did_auth_account_id.to_ss58check_with_version(38u16.into());
    let cli = app_state.kilt_api.lock()?;
    get_did_doc(&addr, &cli).await?;
    Ok(HttpResponse::Ok().json(json!({ "did": format!("did:kilt:{}", addr) })))
}

pub async fn register_device_did(
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ServerError> {
    let keys = app_state.key_manager.lock()?;
    let did_auth_signer = keys.get_did_auth_signer();
    let submitter_signer = keys.get_payment_account_signer();
    let cli = app_state.kilt_api.lock()?;
    let extrinsic_hash = create_did(did_auth_signer.into(), submitter_signer.into(), &cli).await?;
    let formatted_extrinsic_hash = format!("0x{}", hex::encode(extrinsic_hash));
    log::info!("Tx hash: {}", formatted_extrinsic_hash);
    Ok(HttpResponse::Ok().json(json!({ "tx": formatted_extrinsic_hash })))
}

pub async fn submit_extrinsic(
    app_state: web::Data<AppState>,
    body: web::Json<String>,
) -> Result<impl Responder, ServerError> {
    let cli = app_state.kilt_api.lock()?;
    let keys = app_state.key_manager.lock()?;
    let signer = BoxSigner(keys.get_payment_account_signer());
    let call_string = body.0;

    let trimmed_call = call_string.trim_start_matches("0x");

    let call = hex::decode(trimmed_call)
        .map_err(|e| ServerError::Tx(TxError::Format(FormatError::Hex(e))))?;

    let tx_hash = submit_call(&cli, &signer, &call, WaitFor::Finalized).await?;

    log::info!("Tx hash: {}", tx_hash);
    Ok(HttpResponse::Ok().json(json!({ "tx": tx_hash })))
}

pub async fn get_base_claim() -> Result<impl Responder, ServerError> {
    let claim = get_claim_content()?;
    Ok(HttpResponse::Ok().json(json!(claim)))
}

pub async fn post_base_claim(
    body: web::Json<Credential>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ServerError> {
    let base_claim = body.0;

    log::info!("Base claim posted: {:?}", base_claim);

    let key_manager: std::sync::MutexGuard<'_, crate::device::key_manager::PairKeyManager> =
        app_state.key_manager.lock()?;

    let sign_pair = key_manager.get_did_auth_signer();
    let cli = app_state.kilt_api.lock()?;

    let mut jwt_token = app_state.jwt_token.lock()?;

    let is_jwt_healty = check_jwt_health(&jwt_token);

    if !is_jwt_healty {
        let new_token = login_to_open_did(
            &cli,
            sign_pair,
            &app_state.auth_client_id,
            &app_state.auth_endpoint,
        )
        .await?;
        *jwt_token = new_token;
    }

    post_claim_to_attester(&jwt_token, &base_claim, &app_state.attester_endpoint).await?;

    save_claim_content(&base_claim)?;

    Ok(HttpResponse::Ok().json(json!({ "base_claim": base_claim })))
}

pub async fn reset(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let manager = crate::device::reset_keys()?;

    log::info!("new Did: {:?}", manager.get_did_auth_signer().account_id());

    let remove_file = std::fs::remove_file("base_claim.json");
    if remove_file.is_err() {
        log::info!("No claim to delete");
    }

    let mut app_manager = app_state.key_manager.lock()?;
    *app_manager = manager;
    Ok(HttpResponse::Ok())
}

pub async fn get_credential(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let key_manager = app_state.key_manager.lock()?;
    let sign_pair = key_manager.get_did_auth_signer();
    let cli = app_state.kilt_api.lock()?;

    let mut jwt_token = app_state.jwt_token.lock()?;

    let is_jwt_healty = check_jwt_health(&jwt_token);

    if !is_jwt_healty {
        let new_token = login_to_open_did(
            &cli,
            sign_pair,
            &app_state.auth_client_id,
            &app_state.auth_endpoint,
        )
        .await?;
        *jwt_token = new_token;
    }

    let data = get_credentials_from_attester(&jwt_token, &app_state.attester_endpoint).await?;

    Ok(HttpResponse::Ok().json(data))
}
