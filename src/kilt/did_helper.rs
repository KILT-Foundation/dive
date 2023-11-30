use sodiumoxide::crypto::box_::PublicKey;
use sp_core::H256;
use std::str::FromStr;
use subxt::OnlineClient;

use crate::kilt::{
    error::{DidError, FormatError, TxError},
    runtime::{
        runtime_types,
        runtime_types::did::did_details::{DidEncryptionKey, DidPublicKey, DidPublicKeyDetails},
        storage,
    },
    KiltConfig,
};

pub async fn get_encryption_key_from_fulldid_key_uri(
    key_uri: &str,
    cli: &OnlineClient<KiltConfig>,
) -> Result<PublicKey, TxError> {
    let (did, key_id) = parse_key_uri(key_uri)?;

    let decoded_key_id = decode_key_id(&key_id)?;
    let kid = H256::from(decoded_key_id);

    let doc = get_did_doc(&did, cli).await?;

    match find_public_key_by_kid(&kid, &doc.public_keys.0) {
        Some(details) => {
            let pk = match details.key {
                DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519(pk)) => pk,
                _ => return Err(TxError::Did(DidError::InvalidKey)),
            };
            PublicKey::from_slice(&pk).ok_or(TxError::Did(DidError::InvalidKey))
        }
        None => Err(TxError::Did(DidError::MissingKey)),
    }
}

fn parse_key_uri(key_uri: &str) -> Result<(String, String), TxError> {
    let key_uri_parts: Vec<&str> = key_uri.split('#').collect();
    if key_uri_parts.len() != 2 {
        return Err(TxError::Format(FormatError::KeyUri(key_uri.to_string())));
    }
    Ok((key_uri_parts[0].to_string(), key_uri_parts[1].to_string()))
}

fn decode_key_id(key_id: &str) -> Result<[u8; 32], TxError> {
    hex::decode(key_id.trim_start_matches("0x"))
        .map_err(|e| TxError::Hex(e))
        .and_then(|decoded| {
            decoded
                .try_into()
                .map_err(|_| TxError::Format(FormatError::Convert(key_id.to_string())))
        })
}

fn find_public_key_by_kid<'a>(
    kid: &'a H256,
    public_keys: &'a [(H256, DidPublicKeyDetails<u64>)],
) -> Option<&'a DidPublicKeyDetails<u64>> {
    public_keys
        .iter()
        .find(|&(k, _v)| *k == *kid)
        .map(|(_, details)| details)
}
pub async fn get_w3n(did: &str, cli: &OnlineClient<KiltConfig>) -> Result<String, TxError> {
    let account_id = subxt::utils::AccountId32::from_str(did.trim_start_matches("did:kilt:"))
        .map_err(|e| TxError::Format(FormatError::Ss58(e.to_string())))?;
    let storage_key = storage().web3_names().names(account_id);
    let name = cli.storage().at_latest().await?.fetch(&storage_key).await?;

    if let Some(name) = name {
        Ok(String::from_utf8(name.0 .0)
            .map_err(|e| TxError::Format(FormatError::Ss58(e.to_string())))?)
    } else {
        Ok("".into())
    }
}

pub async fn get_did_doc(
    did_input: &str,
    cli: &OnlineClient<KiltConfig>,
) -> Result<runtime_types::did::did_details::DidDetails, TxError> {
    let did = subxt::utils::AccountId32::from_str(did_input.trim_start_matches("did:kilt:"))
        .map_err(|_| TxError::Did(DidError::Format(did_input.to_string())))?;
    let did_doc_key = storage().did().did(&did);
    let details = cli
        .storage()
        .at_latest()
        .await?
        .fetch(&did_doc_key)
        .await?
        .ok_or(TxError::Did(DidError::NotFound(did_input.to_string())))?;

    Ok(details)
}
