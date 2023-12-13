#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Claim {
    #[serde(rename = "cTypeHash")]
    pub ctype_hash: String,
    contents: serde_json::Value,
    pub owner: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub claim: Claim,
    claim_nonce_map: serde_json::Value,
    claim_hashes: Vec<String>,
    delegation_id: Option<String>,
    legitimations: Option<Vec<Credential>>,
    pub root_hash: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct JWTHeader {
    pub alg: String,
    pub typ: String,
    pub kid: String,
    pub crv: String,
    pub kty: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct JWTBody {
    pub iss: String,
    pub sub: String,
    pub nonce: String,
    pub exp: i64,
    pub nbf: i64,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DidAddress {
    pub did: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TxResponse {
    pub tx: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PayerAddress {
    pub address: String,
}
