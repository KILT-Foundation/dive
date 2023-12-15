use sp_core::H256;
use std::str::FromStr;
use subxt::{
    ext::{codec::Encode, sp_core::sr25519::Pair, sp_runtime::MultiSignature},
    tx::{PairSigner, Signer},
    OnlineClient,
};

use crate::kilt::{
    error::{DidError, TxError},
    runtime::{runtime_types, storage},
    KiltConfig,
};

use super::runtime::{
    self,
    runtime_types::{
        bounded_collections::bounded_btree_set::BoundedBTreeSet,
        did::did_details::{DidCreationDetails, DidSignature},
        sp_core::{ecdsa, ed25519, sr25519},
    },
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

pub async fn create_did(
    did_auth_signer: &PairSigner<KiltConfig, Pair>,
    submitter_signer: &PairSigner<KiltConfig, Pair>,
    chain_client: &OnlineClient<KiltConfig>,
) -> Result<H256, TxError> {
    let details = DidCreationDetails {
        did: did_auth_signer.account_id().clone().into(),
        submitter: submitter_signer.account_id().clone().into(),
        new_key_agreement_keys: BoundedBTreeSet(vec![]),
        new_attestation_key: None,
        new_delegation_key: None,
        new_service_details: vec![],
        __subxt_unused_type_params: std::marker::PhantomData,
    };
    let signature = did_auth_signer.sign(&details.encode());
    let did_sig = match signature {
        MultiSignature::Sr25519(sig) => DidSignature::Sr25519(sr25519::Signature(sig.0)),
        MultiSignature::Ed25519(sig) => DidSignature::Ed25519(ed25519::Signature(sig.0)),
        MultiSignature::Ecdsa(sig) => DidSignature::Ecdsa(ecdsa::Signature(sig.0)),
    };
    let tx = runtime::tx().did().create(details, did_sig);
    let events = chain_client
        .tx()
        .sign_and_submit_then_watch_default(&tx, submitter_signer)
        .await?
        .wait_for_finalized_success()
        .await?;
    Ok(events.extrinsic_hash())
}
