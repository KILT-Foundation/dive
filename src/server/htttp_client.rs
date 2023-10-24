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

use super::{
    consts::{ATTESTER_ENDPOINT, CLIENT_ID, OPEN_DID_ENDPOINT},
    Error,
};
use crate::kilt::{get_did_doc, KiltConfig};

#[derive(serde::Deserialize, serde::Serialize)]
struct JWTHeader {
    alg: String,
    typ: String,
    key_uri: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct JWTBody {
    iss: String,
    sub: String,
    nonce: String,
}

pub fn hex_encode<T: AsRef<[u8]>>(data: T) -> String {
    format!("0x{}", hex::encode(data.as_ref()))
}

pub async fn request_login() -> Result<(reqwest::Client, String), Error> {
    let nonce: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let state: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let request_url = format!("/api/v1/authorize?response_type=id_token&client_id=${}&redirect_uri=http://localhost:1606/callback.html&scope=openid&state=${}&nonce=${}", CLIENT_ID, state, nonce );

    let url = format!("{}{}", OPEN_DID_ENDPOINT, request_url);

    let client = reqwest::Client::builder().cookie_store(true).build()?;

    client.get(url).send().await?;

    Ok((client, nonce))
}

pub async fn login_to_open_did(
    cli: &OnlineClient<KiltConfig>,
    signer: Box<dyn Signer<KiltConfig>>,
) -> Result<String, Error> {
    let (client, nonce) = request_login().await?;
    let did_auth_account_id = signer.account_id();
    let did = did_auth_account_id.to_ss58check_with_version(38u16.into());
    let did_doc = get_did_doc(&did, cli).await?;

    let key_uri = did_doc.authentication_key.0;

    let jwt_header = JWTHeader {
        alg: "EdDSA".to_string(),
        typ: "JWT".to_string(),
        key_uri: String::from_utf8(key_uri.to_vec())?,
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
        MultiSignature::Sr25519(sig) => String::from_utf8(sig.0.to_vec()),
        MultiSignature::Ed25519(sig) => String::from_utf8(sig.0.to_vec()),
        MultiSignature::Ecdsa(sig) => String::from_utf8(sig.0.to_vec()),
    }
    .map_err(|_| Error::Unknown)?;

    let jwt_signature_encoded = general_purpose::STANDARD.encode(jwt_signature_string);

    let final_token = format!(
        "{}.{}.{}",
        jwt_header_encoded, jwt_body_encoded, jwt_signature_encoded,
    );

    let request_url = format!("/api/v1/did/{}", final_token);

    let url = format!("{}{}", OPEN_DID_ENDPOINT, request_url);

    let res = client.post(url).send().await?;

    if res.status() == reqwest::StatusCode::BAD_REQUEST {
        log::error!("Bad Request");
    }

    if res.status() == reqwest::StatusCode::NO_CONTENT {
        log::info!("Worked as expected");
        let location = res.headers().get("Location").ok_or(Error::Unknown)?;
        let url_response = location.to_str().map_err(|_| Error::Unknown)?;
        let parsed_url = Url::parse(url_response).map_err(|_| Error::Unknown)?;
        let jwt_token = parsed_url
            .query_pairs()
            .find(|(key, _)| key == "token")
            .ok_or(Error::Unknown)?;
        let token_value = jwt_token.1.to_string();
        return Ok(token_value);
    } else {
        return Err(Error::Unknown);
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AttestationRequest {
    pub ctype_hash: String,
    pub claim: serde_json::Value,
    pub claimer: String,
}

pub async fn post_claim_to_attester(jwt_token: String, base_claim: String) -> Result<(), Error> {
    let mut headers = reqwest::header::HeaderMap::new();
    let auth_header_value = format!("Bearer {}", jwt_token);

    headers.insert(
        AUTHORIZATION,
        auth_header_value.parse().map_err(|_| Error::Unknown)?,
    );

    headers.insert(
        CONTENT_TYPE,
        "application/json".parse().map_err(|_| Error::Unknown)?,
    );

    let base_claim_json = serde_json::to_value(&base_claim)?;

    let request_body = AttestationRequest {
        claim: base_claim_json,
        ctype_hash: "placeholder".to_string(),
        claimer: "placeholder".to_string(),
    };

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let url = format!("{}/api/v1/attestation_request", ATTESTER_ENDPOINT,);

    let response = client.post(url).json(&request_body).send().await?;

    if response.status() == StatusCode::OK {
        log::info!("Requested attestation");
        return Ok(());
    } else {
        log::info!("did not worked");
        return Err(Error::Unknown);
    }
}
