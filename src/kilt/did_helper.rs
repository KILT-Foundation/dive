use base58::FromBase58;
use serde_with::{serde_as, Bytes};
use sodiumoxide::crypto::box_;
use std::str::FromStr;
use subxt::OnlineClient;

use crate::kilt::{
    error::{CredentialAPIError, DidError, TxError},
    runtime::{
        self, runtime_types,
        runtime_types::did::did_details::{DidDetails, DidEncryptionKey, DidPublicKey},
        storage,
    },
    KiltConfig,
};

pub const DID_PREFIX: &'static str = "did:kilt:";
pub const ADDRESS_FORMAT: u16 = 38;

pub async fn query_did_doc(
    did_input: &str,
    chain_client: &OnlineClient<KiltConfig>,
) -> Result<runtime_types::did::did_details::DidDetails, TxError> {
    let did = subxt::utils::AccountId32::from_str(did_input.trim_start_matches(DID_PREFIX))
        .map_err(|_| TxError::Did(DidError::Format(did_input.to_string())))?;
    let did_doc_key = storage().did().did(&did);
    let details = chain_client
        .storage()
        .at_latest()
        .await?
        .fetch(&did_doc_key)
        .await?
        .ok_or(TxError::Did(DidError::NotFound(did_input.to_string())))?;

    Ok(details)
}

#[serde_as]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct LightDidKeyDetails {
    #[serde_as(as = "Bytes")]
    #[serde(rename = "publicKey")]
    public_key: Vec<u8>,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct LightDID {
    e: LightDidKeyDetails,
}

pub fn parse_encryption_key_from_lightdid(
    did: &str,
) -> Result<box_::PublicKey, CredentialAPIError> {
    // example did:kilt:light:00${authAddress}:${details}#encryption
    let first = did
        .split('#')
        .next()
        .ok_or(CredentialAPIError::LightDID("malformed"))?;

    let details = first
        .split(':')
        .skip(4)
        .next()
        .ok_or(CredentialAPIError::LightDID("malformed"))?;

    let bs: Vec<u8> = details
        .chars()
        .skip(1)
        .collect::<String>()
        .from_base58()
        .map_err(|_| CredentialAPIError::LightDID("malformed base58"))?;

    let light_did: LightDID = serde_cbor::from_slice(&bs[1..])
        .map_err(|_| CredentialAPIError::LightDID("Deserialization"))?;
    box_::PublicKey::from_slice(&light_did.e.public_key)
        .ok_or(CredentialAPIError::LightDID("Not a valid public key"))
}

pub async fn get_did_doc(
    did: &str,
    cli: &OnlineClient<KiltConfig>,
) -> Result<DidDetails, CredentialAPIError> {
    let did = subxt::utils::AccountId32::from_str(did.trim_start_matches("did:kilt:"))
        .map_err(|_| CredentialAPIError::Did("Invalid DID"))?;
    let did_doc_key = runtime::storage().did().did(&did);
    let details = cli
        .storage()
        .at_latest()
        .await?
        .fetch(&did_doc_key)
        .await?
        .ok_or(CredentialAPIError::Did("DID not found"))?;

    Ok(details)
}

fn parse_key_uri(key_uri: &str) -> Result<(&str, sp_core::H256), CredentialAPIError> {
    let key_uri_parts: Vec<&str> = key_uri.split('#').collect();
    if key_uri_parts.len() != 2 {
        return Err(CredentialAPIError::Did("Invalid sender key URI"));
    }
    let did = key_uri_parts[0];
    let key_id = key_uri_parts[1];
    let kid_bs: [u8; 32] = hex::decode(key_id.trim_start_matches("0x"))
        .map_err(|_| CredentialAPIError::Did("key ID isn't valid hex"))?
        .try_into()
        .map_err(|_| CredentialAPIError::Did("key ID is expected to have 32 bytes"))?;
    let kid = sp_core::H256::from(kid_bs);

    Ok((did, kid))
}

pub async fn get_encryption_key_from_fulldid_key_uri(
    key_uri: &str,
    chain_client: &OnlineClient<KiltConfig>,
) -> Result<box_::PublicKey, CredentialAPIError> {
    let (did, kid) = parse_key_uri(key_uri)?;
    let doc = get_did_doc(did, chain_client).await?;

    let (_, details) = doc
        .public_keys
        .0
        .iter()
        .find(|&(k, _v)| *k == kid)
        .ok_or(CredentialAPIError::Did("Could not get sender public key"))?;
    let pk = if let DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(pk)) = details.key {
        pk
    } else {
        return Err(CredentialAPIError::Did("Invalid sender public key"));
    };
    box_::PublicKey::from_slice(&pk).ok_or(CredentialAPIError::Did("Invalid sender public key"))
}

#[serde_as]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct LightDidKeyDetails {
    #[serde_as(as = "Bytes")]
    #[serde(rename = "publicKey")]
    public_key: Vec<u8>,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct LightDIDDetails {
    e: LightDidKeyDetails,
}

pub fn parse_encryption_key_from_lightdid(did: &str) -> Result<box_::PublicKey, ServerError> {
    // example did:kilt:light:00${authAddress}:${details}#encryption
    let mut parts = did.split('#');
    let first = parts.next().ok_or(ServerError::LightDID("malformed"))?;
    let mut parts = first.split(':').skip(4);
    let details = parts.next().ok_or(ServerError::LightDID("malformed"))?;

    let mut chars = details.chars();
    chars.next().ok_or(ServerError::LightDID("malformed"))?;
    let bs: Vec<u8> = FromBase58::from_base58(chars.as_str())
        .map_err(|_| ServerError::LightDID("malformed base58"))?;

    let details: LightDIDDetails =
        serde_cbor::from_slice(&bs[1..]).map_err(|_| ServerError::LightDID("Deserialization"))?;
    box_::PublicKey::from_slice(&details.e.public_key)
        .ok_or(ServerError::LightDID("Not a valid public key"))
}
