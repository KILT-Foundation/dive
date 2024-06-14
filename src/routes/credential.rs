use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use sodiumoxide::crypto::box_;
use sp_core::H256;

use crate::{
    device::key_manager::KeyManager,
    error::ServerError,
    http_client::{check_jwt_health, get_credentials_from_attester, login_to_open_did},
    kilt::{connect, error::CredentialAPIError},
    routes::dto::*,
    AppState,
};

#[get("")]
async fn get_credential(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let key_manager = app_state.key_manager.lock().await;
    let sign_pair = key_manager.get_did_auth_signer();
    let chain_client = connect(&app_state.wss_endpoint).await?;

    let mut jwt_token = app_state.jwt_token.lock().await;

    let is_jwt_healty = check_jwt_health(&jwt_token);

    if !is_jwt_healty {
        let new_token = login_to_open_did(
            &chain_client,
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

#[post("/terms")]
async fn get_terms(
    state: web::Data<AppState>,
    session: Session,
    claim: web::Json<Claim>,
) -> Result<HttpResponse, ServerError> {
    let sender_key_uri = session
        .get::<String>("encryption_key_uri")
        .map_err(|_| CredentialAPIError::Challenge("Session not set"))?
        .ok_or(CredentialAPIError::Challenge("Session not set"))?;

    let others_pubkey =
        crate::kilt::did_helper::parse_encryption_key_from_lightdid(&sender_key_uri)?;

    let encryption_key_uri = state.session_encryption_public_key_uri.clone();

    let sender = encryption_key_uri
        .split('#')
        .collect::<Vec<&str>>()
        .first()
        .ok_or_else(|| CredentialAPIError::Attestation("Invalid Key URI for sender"))?
        .to_owned();

    let content = SubmitTermsMessageContent {
        c_types: vec![claim.0.ctype_hash.clone()],
        claim: claim.0,
        legitimations: Some(vec![]),
    };

    let msg = Message {
        body: MessageBody {
            content,
            type_: "submit-terms".to_string(),
        },
        created_at: 0,
        sender: sender.to_string(),
        receiver: sender_key_uri.clone(),
        message_id: uuid::Uuid::new_v4().to_string(),
        in_reply_to: None,
        references: None,
    };

    let msg_json = serde_json::to_string(&msg).unwrap();
    let msg_bytes = msg_json.as_bytes();
    let our_secretkey = state.secret_key.clone();
    let nonce = box_::gen_nonce();
    let encrypted_msg = box_::seal(msg_bytes, &nonce, &others_pubkey, &our_secretkey);
    let response = EncryptedMessage {
        cipher_text: encrypted_msg,
        nonce,
        sender_key_uri: encryption_key_uri.clone(),
        receiver_key_uri: sender_key_uri,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[post("")]
async fn request_attestation(
    app_state: web::Data<AppState>,
    encrypted_message: web::Json<EncryptedMessage>,
) -> Result<HttpResponse, ServerError> {
    let chain_client = connect(&app_state.wss_endpoint).await?;

    let others_pubkey = crate::kilt::did_helper::get_encryption_key_from_fulldid_key_uri(
        &encrypted_message.sender_key_uri,
        &chain_client,
    )
    .await?;

    let decrypted_message_bytes = box_::open(
        &encrypted_message.cipher_text,
        &encrypted_message.nonce,
        &others_pubkey,
        &app_state.secret_key,
    )
    .map_err(|_| CredentialAPIError::Attestation("Unable to decrypt"))?;

    let decrypted_message: Message<RequestAttestationMessageContent> =
        serde_json::from_slice(&decrypted_message_bytes).unwrap();

    let credential = decrypted_message.body.content.credential;

    let ctype_hash = hex::decode(credential.claim.ctype_hash.trim_start_matches("0x").trim())?;
    let claim_hash = hex::decode(credential.root_hash.trim_start_matches("0x").trim())?;
    if claim_hash.len() != 32 || ctype_hash.len() != 32 {
        Err(actix_web::error::ErrorBadRequest(
            "Claim hash or ctype hash have a wrong format",
        ))?
    }

    let payer = app_state
        .key_manager
        .lock()
        .await
        .get_payment_account_signer();

    crate::kilt::tx::create_claim(
        H256::from_slice(&claim_hash),
        H256::from_slice(&ctype_hash),
        &app_state.did_attester,
        &chain_client,
        &payer,
        &app_state.signer,
    )
    .await?;

    Ok(HttpResponse::Ok().json("ok"))
}

pub fn get_credential_scope() -> Scope {
    web::scope("/api/v1/credential")
        .service(get_credential)
        .service(get_terms)
        .service(request_attestation)
}
