use actix_web::{web, HttpResponse, Responder};

use crate::{
    device::{
        file_manager::{get_claim_content, save_claim_content},
        key_manager::KeyManager,
    },
    dto::{Credential, DidAddress, PayerAddress, TxResponse},
    error::ServerError,
    http_client::{
        check_jwt_health, get_credentials_from_attester, login_to_open_did, post_claim_to_attester,
    },
    kilt::{
        did_helper::{create_did, query_did_doc, DID_PREFIX},
        error::{FormatError, TxError},
        tx::{submit_call, BoxSigner, WaitFor},
    },
    AppState,
};

pub async fn get_payment_account_address(
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ServerError> {
    let address = app_state.payment_addr.clone();
    Ok(HttpResponse::Ok().json(PayerAddress { address }))
}

pub async fn get_did(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let addr = &app_state.did_addr;
    let chain_client = &app_state.chain_client;
    query_did_doc(&addr, chain_client).await?;
    Ok(HttpResponse::Ok().json(DidAddress {
        did: format!("{}{}", DID_PREFIX, addr),
    }))
}

pub async fn register_device_did(
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ServerError> {
    let keys = app_state.key_manager.lock().await;
    let did_auth_signer = keys.get_did_auth_signer();
    let submitter_signer = keys.get_payment_account_signer();
    let chain_client = &app_state.chain_client;
    let extrinsic_hash = create_did(
        did_auth_signer.into(),
        submitter_signer.into(),
        chain_client,
    )
    .await?;
    let tx = format!("0x{}", hex::encode(extrinsic_hash));
    log::info!("Tx hash: {}", tx);
    Ok(HttpResponse::Ok().json(TxResponse { tx }))
}

pub async fn submit_extrinsic(
    app_state: web::Data<AppState>,
    body: web::Json<String>,
) -> Result<impl Responder, ServerError> {
    let chain_client = &app_state.chain_client;
    let keys = app_state.key_manager.lock().await;
    let signer = BoxSigner(keys.get_payment_account_signer());
    let call_string = body.0;

    let trimmed_call = call_string.trim_start_matches("0x");

    let call = hex::decode(trimmed_call)
        .map_err(|e| ServerError::Tx(TxError::Format(FormatError::Hex(e))))?;

    let tx = submit_call(chain_client, &signer, &call, WaitFor::Finalized).await?;

    log::info!("Tx hash: {}", tx);
    Ok(HttpResponse::Ok().json(TxResponse { tx }))
}

pub async fn get_base_claim() -> Result<impl Responder, ServerError> {
    let claim = get_claim_content()?;
    Ok(HttpResponse::Ok().json(claim))
}

pub async fn post_base_claim(
    body: web::Json<Credential>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ServerError> {
    let base_claim = body.0;

    log::info!("Base claim posted: {:?}", base_claim);

    let key_manager = app_state.key_manager.lock().await;

    let sign_pair = key_manager.get_did_auth_signer();
    let chain_client = &app_state.chain_client;

    let mut jwt_token = app_state.jwt_token.lock().await;

    let is_jwt_healty = check_jwt_health(&jwt_token);

    if !is_jwt_healty {
        let new_token = login_to_open_did(
            chain_client,
            sign_pair,
            &app_state.auth_client_id,
            &app_state.auth_endpoint,
            &app_state.redirect_url,
        )
        .await?;
        *jwt_token = new_token;
    }

    post_claim_to_attester(&jwt_token, &base_claim, &app_state.attester_endpoint).await?;

    save_claim_content(&base_claim)?;

    Ok(HttpResponse::Ok().json(base_claim))
}

pub async fn reset(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let new_key_manager = crate::device::reset_did_keys()?;

    log::info!(
        "new Did: {:?}",
        new_key_manager.get_did_auth_signer().account_id()
    );

    let remove_file = tokio::fs::remove_file("base_claim.json").await;

    if remove_file.is_err() {
        let err = remove_file.unwrap_err();
        log::info!("{}", err.to_string());
    }

    let mut key_manager = app_state.key_manager.lock().await;
    *key_manager = new_key_manager;
    Ok(HttpResponse::Ok())
}

pub async fn get_credential(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let key_manager = app_state.key_manager.lock().await;
    let sign_pair = key_manager.get_did_auth_signer();
    let chain_client = &app_state.chain_client;

    let mut jwt_token = app_state.jwt_token.lock().await;

    let is_jwt_healty = check_jwt_health(&jwt_token);

    if !is_jwt_healty {
        let new_token = login_to_open_did(
            chain_client,
            sign_pair,
            &app_state.auth_client_id,
            &app_state.auth_endpoint,
            &app_state.redirect_url,
        )
        .await?;
        *jwt_token = new_token;
    }

    let data = get_credentials_from_attester(&jwt_token, &app_state.attester_endpoint).await?;

    Ok(HttpResponse::Ok().json(data))
}
