use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Scope};
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_;
use uuid::Uuid;

use crate::{
    error::ServerError,
    utils::{hex_nonce, prefixed_hex},
    AppState,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChallengeData {
    #[serde(rename = "dAppName")]
    pub app_name: String,
    #[serde(rename = "dAppEncryptionKeyUri")]
    pub encryption_key_uri: String,
    pub challenge: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeResponse {
    pub encryption_key_uri: String,
    #[serde(with = "prefixed_hex")]
    pub encrypted_challenge: Vec<u8>,
    #[serde(with = "hex_nonce")]
    pub nonce: box_::Nonce,
}

#[get("")]
async fn challenge_handler(
    state: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse, ServerError> {
    let app_name = state.app_name.clone();

    let encryption_key_uri = state.key_uri.clone();

    let challenge = Uuid::new_v4().as_bytes().to_vec();

    session
        .insert("challenge", challenge.clone())
        .map_err(|_| ServerError::Challenge("Could not insert encryption key"))?;

    let challenge_data = ChallengeData {
        challenge,
        app_name,
        encryption_key_uri,
    };

    Ok(HttpResponse::Ok().json(challenge_data))
}

#[post("")]
async fn challenge_response_handler(
    state: web::Data<AppState>,
    challenge_response: web::Json<ChallengeResponse>,
    session: Session,
) -> Result<HttpResponse, ServerError> {
    let challenge = session
        .get::<Vec<u8>>("challenge")
        .map_err(|_| ServerError::Challenge("Session not set"))?
        .ok_or(ServerError::Challenge("Session not set"))?;

    let encryption_key_uri = &challenge_response.encryption_key_uri;
    let others_pubkey =
        crate::kilt::did_helper::parse_encryption_key_from_lightdid(encryption_key_uri)?;

    let decrypted_challenge = box_::open(
        &challenge_response.encrypted_challenge,
        &challenge_response.nonce,
        &others_pubkey,
        &state.secret_key,
    )
    .map_err(|_| ServerError::Challenge("Unable to decrypt"))?;

    if decrypted_challenge == challenge {
        return Err(ServerError::Challenge("Challenge do not match"));
    }

    session
        .insert("encryption_key_uri", encryption_key_uri)
        .map_err(|_| ServerError::Challenge("Could not insert encryption key"))?;

    Ok(HttpResponse::Ok().json("Ok"))
}

pub fn get_challenge_scope() -> Scope {
    web::scope("/api/v1/challenge")
        .service(challenge_handler)
        .service(challenge_response_handler)
}
