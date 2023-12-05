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

use crate::{
    dto::*,
    error::ServerError,
    kilt::{did_helper::get_did_doc, KiltConfig},
};

fn hex_encode<T: AsRef<[u8]>>(data: T) -> String {
    format!("0x{}", hex::encode(data.as_ref()))
}

fn is_jwt_token_not_expired(jwt_token: &str) -> bool {
    let parts: Vec<&str> = jwt_token.split('.').collect();
    let Some(header_option) = parts.get(1) else { return false };
    let Ok(decoded_header)  = general_purpose::STANDARD.decode(header_option) else { return false };
    let Ok(jwt_header) = serde_json::from_slice::<serde_json::Value>(&decoded_header) else { return false };

    let Ok(expired) = serde_json::from_value::<i64>(jwt_header["exp"].clone()) else { return false };
    let now = chrono::Utc::now().timestamp();
    expired > now
}

fn is_jwt_token_set(jwt_token: &str) -> bool {
    !jwt_token.is_empty()
}

pub fn check_jwt_health(jwt_token: &str) -> bool {
    is_jwt_token_not_expired(jwt_token) && is_jwt_token_set(jwt_token)
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

    let request_url = Url::parse_with_params(
        &format!("{}/api/v1/authorize", auth_endpoint),
        &[
            ("response_type", "id_token"),
            ("client_id", client_id),
            // TODO may be as a env variable?
            ("redirect_uri", "http://localhost:3333"),
            ("scope", "openid"),
            ("state", &state),
            ("nonce", &nonce),
        ],
    )
    .map_err(ServerError::from)?;

    let client = reqwest::Client::builder().cookie_store(true).build()?;

    client.get(request_url.as_str()).send().await?;

    Ok((client, nonce))
}

fn get_id_token(url_str: &str) -> Result<String, ServerError> {
    let url = Url::parse(url_str).map_err(ServerError::from)?;

    if let Some(fragment) = url.fragment() {
        if let Some(token) = fragment
            .split('&')
            .find(|segment| segment.starts_with("id_token="))
            .map(|segment| &segment[9..])
        {
            return Ok(token.to_string());
        }
    }

    Err(ServerError::Unknown)
}

fn get_encoded_jwt_parts(
    did: String,
    key_uri: String,
    nonce: String,
) -> Result<(String, String), ServerError> {
    let jwt_header = JWTHeader {
        alg: "EdDSA".to_string(),
        typ: "JWT".to_string(),
        kid: key_uri,
        crv: "ed25519".to_string(),
        kty: "ed25519".to_string(),
    };

    let jwt_body = JWTBody {
        iss: did.clone(),
        sub: did.clone(),
        nonce,
        nbf: chrono::Utc::now().timestamp(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    };

    let jwt_header_string = serde_json::to_string(&jwt_header)?;
    let jwt_body_string = serde_json::to_string(&jwt_body)?;

    let jwt_header_encoded = general_purpose::STANDARD.encode(jwt_header_string);
    let jwt_body_encoded = general_purpose::STANDARD.encode(jwt_body_string);

    Ok((jwt_header_encoded, jwt_body_encoded))
}

fn get_encoded_jwt_signature(
    jwt_header_encoded: &str,
    jwt_body_encoded: &str,
    signer: Box<dyn Signer<KiltConfig>>,
) -> String {
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

    general_purpose::STANDARD.encode(jwt_signature_string)
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

    let kid = hex_encode(did_doc.authentication_key.as_bytes());
    let key_uri = format!("{}#{}", did, kid);

    let (jwt_header_encoded, jwt_body_encoded) = get_encoded_jwt_parts(did, key_uri, nonce)?;

    let jwt_signature_encoded =
        get_encoded_jwt_signature(&jwt_header_encoded, &jwt_body_encoded, signer);

    let final_token = format!(
        "{}.{}.{}",
        jwt_header_encoded, jwt_body_encoded, jwt_signature_encoded,
    );

    let request_url = format!("/api/v1/did/{}", final_token);

    let url = format!("{}{}", auth_endpoint, request_url);

    let res = client.post(url).send().await?;

    if res.status() == reqwest::StatusCode::UNAUTHORIZED {
        log::error!("Unauthorized Error while trying to login in OpenDid");
        return Err(ServerError::Unknown);
    }

    if res.status() == reqwest::StatusCode::NO_CONTENT {
        let location = res.headers().get("Location").ok_or(ServerError::Unknown)?;
        let url_response = location.to_str().map_err(|_| ServerError::Unknown)?;
        let token = get_id_token(url_response)?;

        log::info!("Successfully login in by openDid. New token: {}", token);
        return Ok(token);
    }

    return Err(ServerError::Unknown);
}

pub async fn post_claim_to_attester(
    jwt_token: &str,
    base_claim: &Credential,
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

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let url = format!("{}/api/v1/attestation_request", attester_url);

    let response = client.post(url).json(base_claim).send().await?;

    if response.status() == StatusCode::OK {
        log::info!("Requested attestation");

        return Ok(());
    } else {
        log::info!("did not worked");
        return Err(ServerError::Unknown);
    }
}

pub async fn get_credentials_from_attester(
    jwt_token: &str,
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

    log::info!("Response from attester service {:?}", response);

    let bytes = response.bytes().await?;
    let data: serde_json::Value = serde_json::from_slice(&bytes)?;

    Ok(data)
}
