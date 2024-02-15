use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_::Nonce;
use std::collections::HashMap;

use crate::utils::{hex_nonce, prefixed_hex};

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
    #[serde(rename = "cTypes")]
    pub c_types: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub claim: Claim,
    claim_nonce_map: HashMap<String, String>,
    claim_hashes: Vec<String>,
    delegation_id: Option<String>,
    legitimations: Option<Vec<Credential>>,
    pub root_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestAttestationMessageContent {
    pub credential: Credential,
    pub quote: Option<serde_json::Value>,
}
