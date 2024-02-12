use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_::{self, Nonce};

use crate::{
    device::key_manager::KeyManager,
    error::ServerError,
    http_client::{check_jwt_health, get_credentials_from_attester, login_to_open_did},
    utils::{hex_nonce, prefixed_hex},
    AppState,
};

#[get("")]
async fn get_credential(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Claim {
    #[serde(rename = "cTypeHash")]
    pub ctype_hash: String,
    contents: serde_json::Value,
    pub owner: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageBody<T> {
    #[serde(rename = "type")]
    pub type_: String,
    pub content: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message<T> {
    pub body: MessageBody<T>,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    pub sender: String,
    pub receiver: String,
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "inReplyTo")]
    pub in_reply_to: Option<String>,
    pub references: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    #[serde(rename = "ciphertext")]
    #[serde(with = "prefixed_hex")]
    pub cipher_text: Vec<u8>,
    #[serde(with = "hex_nonce")]
    pub nonce: Nonce,
    #[serde(rename = "receiverKeyUri")]
    pub receiver_key_uri: String,
    #[serde(rename = "senderKeyUri")]
    pub sender_key_uri: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitTermsMessageContent {
    pub claim: Claim,
    pub legitimations: Option<Vec<String>>,
}

#[get("/terms")]
async fn get_terms(
    state: web::Data<AppState>,
    session: Session,
    claim: web::Json<Claim>,
) -> Result<HttpResponse, ServerError> {
    let sender_key_uri = session
        .get::<String>("encryption_key_uri")
        .map_err(|_| ServerError::Challenge("Session not set"))?
        .ok_or(ServerError::Challenge("Session not set"))?;

    let others_pubkey =
        crate::kilt::did_helper::parse_encryption_key_from_lightdid(&sender_key_uri)?;

    let encryption_key_uri = state.session_encryption_public_key_uri.clone();

    let sender = encryption_key_uri
        .split('#')
        .collect::<Vec<&str>>()
        .first()
        .ok_or_else(|| ServerError::Attestation("Invalid Key URI for sender"))?
        .to_owned();

    let content = SubmitTermsMessageContent {
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

pub fn get_credential_scope() -> Scope {
    web::scope("/api/v1/credential")
        .service(get_credential)
        .service(get_terms)
}
