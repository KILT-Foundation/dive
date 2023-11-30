#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Claim {
    #[serde(rename = "cTypeHash")]
    pub ctype_hash: String,
    contents: serde_json::Value,
    pub owner: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub claim: Claim,
    claim_nonce_map: serde_json::Value,
    claim_hashes: Vec<String>,
    delegation_id: Option<String>,
    legitimations: Option<Vec<Credential>>,
    pub root_hash: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AttestationResponse {
    pub id: String,
    pub approved: bool,
    pub revoked: bool,
    pub ctype_hash: String,
    pub credential: serde_json::Value,
    pub claimer: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct JWTHeader {
    pub alg: String,
    pub typ: String,
    pub key_uri: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct JWTBody {
    pub iss: String,
    pub sub: String,
    pub nonce: String,
}
