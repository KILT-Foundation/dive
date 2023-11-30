use actix_web::{web, HttpResponse, Responder};
use futures::StreamExt;
use serde_json::json;
use subxt::ext::{codec::Encode, sp_core::crypto::Ss58Codec, sp_runtime::MultiSignature};

use crate::{
    device::key_manager::KeyManager,
    kilt::{
        runtime::{
            self,
            runtime_types::{
                did::did_details::{DidCreationDetails, DidSignature},
                sp_core::{bounded::bounded_btree_set::BoundedBTreeSet, ecdsa, ed25519, sr25519},
            },
            storage,
        },
        tx::{submit_call, BoxSigner, WaitFor},
    },
    server::error::ServerError,
};

use super::{
    htttp_client::{get_credential_request, login_to_open_did, post_claim_to_attester},
    server::AppState,
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
    let query = storage()
        .did()
        .did(subxt::utils::AccountId32::from(did_auth_account_id));
    let result = cli.storage().at_latest().await?.fetch(&query).await?;

    match result {
        Some(_) => Ok(HttpResponse::Ok().json(json!({ "did": format!("did:kilt:{}", addr) }))),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

pub async fn register_device_did(
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ServerError> {
    let keys = crate::device::init_keys()?;
    let did_auth_signer = keys.get_did_auth_signer();
    let submitter_signer = keys.get_payment_account_signer();
    let details = DidCreationDetails {
        did: did_auth_signer.account_id().into(),
        submitter: submitter_signer.account_id().into(),
        new_key_agreement_keys: BoundedBTreeSet(vec![]),
        new_attestation_key: None,
        new_delegation_key: None,
        new_service_details: vec![],
    };
    let signature = did_auth_signer.sign(&details.encode());
    let did_sig = match signature {
        MultiSignature::Sr25519(sig) => DidSignature::Sr25519(sr25519::Signature(sig.0)),
        MultiSignature::Ed25519(sig) => DidSignature::Ed25519(ed25519::Signature(sig.0)),
        MultiSignature::Ecdsa(sig) => DidSignature::Ecdsa(ecdsa::Signature(sig.0)),
    };
    let tx = runtime::tx().did().create(details, did_sig);
    let api = app_state.kilt_api.lock()?;
    let signer = BoxSigner(submitter_signer);
    let events = api
        .tx()
        .sign_and_submit_then_watch_default(&tx, &signer)
        .await?
        .wait_for_finalized_success()
        .await?;
    println!("events: {:?}", events);
    Ok(HttpResponse::Ok().json(json!({
        "tx": format!("0x{}", hex::encode(events.extrinsic_hash()))
    })))
}

const MAX_BODY_SIZE: usize = 262_144; // max payload size is 256k
pub async fn submit_extrinsic(
    app_state: web::Data<AppState>,
    mut payload: web::Payload,
) -> Result<impl Responder, ServerError> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_BODY_SIZE {
            eprintln!("too big body");
            return Err(ServerError::Unknown);
        }
        body.extend_from_slice(&chunk);
    }

    let cli = app_state.kilt_api.lock()?;
    let keys = app_state.key_manager.lock()?;
    let signer = BoxSigner(keys.get_payment_account_signer());

    let call = hex::decode(String::from_utf8(body.to_vec())?.trim_start_matches("0x"))?;
    println!("call decoded");

    let tx_hash = submit_call(&cli, &signer, &call, WaitFor::Finalized).await?;

    Ok(HttpResponse::Ok().json(json!({ "tx": tx_hash })))
}

pub async fn get_base_claim() -> Result<impl Responder, ServerError> {
    let base_claim = std::fs::read_to_string("base_claim.json")?;
    Ok(HttpResponse::Ok().json(json!({
        "base_claim": base_claim,
        "attested": false,
    })))
}

pub async fn post_base_claim(
    mut payload: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ServerError> {
    let mut body = web::BytesMut::new();

    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_BODY_SIZE {
            eprintln!("too big body");
            return Err(ServerError::Unknown);
        }
        body.extend_from_slice(&chunk);
    }
    let base_claim = String::from_utf8(body.to_vec())?;

    let key_manager: std::sync::MutexGuard<'_, crate::device::key_manager::PairKeyManager> =
        app_state.key_manager.lock()?;

    let sign_pair = key_manager.get_did_auth_signer();
    let cli = app_state.kilt_api.lock()?;

    log::info!("Try to login");

    let jwt_token = login_to_open_did(
        &cli,
        sign_pair,
        &app_state.auth_client_id,
        &app_state.auth_endpoint,
    )
    .await?;

    post_claim_to_attester(jwt_token, base_claim.clone(), &app_state.attester_endpoint).await?;

    std::fs::write("base_claim.json", &base_claim)?;
    Ok(HttpResponse::Ok().json(json!({ "base_claim": base_claim })))
}

pub async fn reset(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let manager = crate::device::reset_did_keys()?;
    let remove_file = std::fs::remove_file("base_claim.json");
    if remove_file.is_err() {
        println!("No claim to delete");
    }
    let mut app_manager = app_state.key_manager.lock().unwrap();
    *app_manager = manager;
    Ok(HttpResponse::Ok())
}

pub async fn get_credential(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let key_manager = app_state.key_manager.lock()?;
    let sign_pair = key_manager.get_did_auth_signer();
    let cli = app_state.kilt_api.lock()?;
    let jwt_token = login_to_open_did(
        &cli,
        sign_pair,
        &app_state.auth_client_id,
        &app_state.auth_client_id,
    )
    .await?;
    let data = get_credential_request(jwt_token, &app_state.attester_endpoint).await?;
    Ok(HttpResponse::Ok().json(data))
}
