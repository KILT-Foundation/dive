use base64::{engine::general_purpose, Engine};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    StatusCode,
};
use sha2::{Digest, Sha512};
use subxt::{
    ext::{sp_core::crypto::Ss58Codec, sp_runtime::MultiSignature},
    tx::Signer,
    OnlineClient,
};
use url::Url;

use super::dto::*;
use crate::{
    kilt::{did_helper::get_did_doc, KiltConfig},
    server::error::ServerError,
};

pub fn hex_encode<T: AsRef<[u8]>>(data: T) -> String {
    format!("0x{}", hex::encode(data.as_ref()))
}

pub async fn request_login(
    client_id: &str,
    auth_endpoint: &str,
) -> Result<(reqwest::Client, String), ServerError> {
    let nonce: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    let state: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    let request_url = format!("/api/v1/authorize?response_type=id_token&client_id={}&redirect_uri=http://localhost:3333&scope=openid&state={}&nonce={}", client_id, state, nonce );

    let url = format!("{}{}", auth_endpoint, request_url);

    let client = reqwest::Client::builder().cookie_store(true).build()?;

    client.get(url).send().await?;

    Ok((client, nonce))
}

fn get_id_token(url_str: &str) -> Result<String, ServerError> {
    if let Ok(url) = Url::parse(url_str) {
        if let Some(fragment) = url.fragment() {
            let id_token = fragment
                .split('&')
                .find(|segment| segment.starts_with("id_token="))
                .map(|segment| &segment[9..]);

            match id_token {
                Some(token) => {
                    return Ok(token.to_string());
                }
                None => {
                    return Err(ServerError::Unknown);
                }
            }
        } else {
            return Err(ServerError::Unknown);
        }
    } else {
        return Err(ServerError::Unknown);
    }
}

pub async fn login_to_open_did(
    cli: &OnlineClient<KiltConfig>,
    signer: Box<dyn Signer<KiltConfig>>,
    client_id: &str,
    auth_endpoint: &str,
) -> Result<String, ServerError> {
    let (client, nonce) = request_login(client_id, auth_endpoint).await?;
    let did_auth_account_id = signer.account_id();
    let did = did_auth_account_id.to_ss58check_with_version(38u16.into());
    let did_doc = get_did_doc(&did, cli).await?;

    let key_uri = hex_encode(did_doc.authentication_key.as_bytes());

    let jwt_header = JWTHeader {
        alg: "EdDSA".to_string(),
        typ: "JWT".to_string(),
        key_uri,
    };

    let jwt_body = JWTBody {
        iss: did.clone(),
        sub: did.clone(),
        nonce,
    };

    let jwt_header_string = serde_json::to_string(&jwt_header)?;
    let jwt_body_string = serde_json::to_string(&jwt_body)?;

    let jwt_header_encoded = general_purpose::STANDARD.encode(jwt_header_string);
    let jwt_body_encoded = general_purpose::STANDARD.encode(jwt_body_string);

    let mut hasher = Sha512::new();
    hasher.update(format!("{}.{}", jwt_header_encoded, jwt_body_encoded));
    let data_to_sign_hex = hex_encode(hasher.finalize());
    let data_to_sign = data_to_sign_hex.trim_start_matches("0x").as_bytes();

    let jwt_signature = signer.sign(data_to_sign);

    let jwt_signature_string = match jwt_signature {
        MultiSignature::Sr25519(sig) => hex_encode(sig.0),
        MultiSignature::Ed25519(sig) => hex_encode(sig.0),
        MultiSignature::Ecdsa(sig) => hex_encode(sig.0),
    };

    let jwt_signature_encoded = general_purpose::STANDARD.encode(jwt_signature_string);

    let final_token = format!(
        "{}.{}.{}",
        jwt_header_encoded, jwt_body_encoded, jwt_signature_encoded,
    );

    let request_url = format!("/api/v1/did/{}", final_token);

    let url = format!("{}{}", auth_endpoint, request_url);

    let res = client.post(url).send().await?;

    if res.status() == reqwest::StatusCode::BAD_REQUEST {
        log::error!("Bad Request");
    }

    if res.status() == reqwest::StatusCode::NO_CONTENT {
        log::info!("Worked as expected");
        let location = res.headers().get("Location").ok_or(ServerError::Unknown)?;
        let url_response = location.to_str().map_err(|_| ServerError::Unknown)?;
        let token = get_id_token(url_response)?;
        return Ok(token);
    } else {
        return Err(ServerError::Unknown);
    }
}

pub async fn post_claim_to_attester(
    jwt_token: String,
    base_claim: String,
    attester_url: &str,
) -> Result<(), ServerError> {
    let mut headers = reqwest::header::HeaderMap::new();

    let auth_header_value = format!("Bearer {}", jwt_token);

    headers.insert(
        AUTHORIZATION,
        auth_header_value
            .parse()
            .map_err(|_| ServerError::Unknown)?,
    );

    headers.insert(
        CONTENT_TYPE,
        "application/json"
            .parse()
            .map_err(|_| ServerError::Unknown)?,
    );

    let base_claim_json = serde_json::from_str::<Credential>(&base_claim)?;

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let url = format!("{}/api/v1/attestation_request", attester_url);

    let response = client.post(url).json(&base_claim_json).send().await?;

    if response.status() == StatusCode::OK {
        log::info!("Requested attestation");

        return Ok(());
    } else {
        log::info!("did not worked");
        return Err(ServerError::Unknown);
    }
}

pub async fn get_credential_request(
    jwt_token: String,
    attester_url: &str,
) -> Result<serde_json::Value, ServerError> {
    let mut headers = reqwest::header::HeaderMap::new();

    let auth_header_value = format!("Bearer {}", jwt_token);

    headers.insert(
        AUTHORIZATION,
        auth_header_value
            .parse()
            .map_err(|_| ServerError::Unknown)?,
    );

    headers.insert(
        CONTENT_TYPE,
        "application/json"
            .parse()
            .map_err(|_| ServerError::Unknown)?,
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let url = format!("{}/api/v1/attestation_request", attester_url);

    let response = client.get(url).send().await?;

    let data = response.json::<serde_json::Value>().await?;

    Ok(data)
}
