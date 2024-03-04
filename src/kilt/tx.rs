use sp_core::H256;
use std::str::FromStr;
use subxt::{ext::sp_core::sr25519::Pair as Sr25519Pair, tx::TxPayload, utils::AccountId32};
use subxt::{
    ext::{codec::Encode, sp_core::sr25519::Pair, sp_runtime::MultiSignature},
    tx::{PairSigner, Signer},
    OnlineClient,
};

use crate::kilt::{
    error::TxError,
    runtime::runtime_types,
    runtime::{
        self,
        runtime_types::{
            bounded_collections::bounded_btree_set::BoundedBTreeSet,
            bounded_collections::bounded_vec::BoundedVec,
            did::did_details::DidAuthorizedCallOperation,
            did::did_details::{DidCreationDetails, DidSignature},
            did::service_endpoints::DidEndpoint,
            sp_core::{ecdsa, ed25519, sr25519},
        },
    },
    utils::{calculate_signature, get_current_block, get_next_tx_counter},
    KiltConfig, RuntimeCall,
};

#[derive(Debug, Clone)]
pub struct RawCall {
    pub call: Vec<u8>,
}

impl TxPayload for RawCall {
    fn encode_call_data_to(
        &self,
        _metadata: &subxt::Metadata,
        out: &mut Vec<u8>,
    ) -> Result<(), subxt::Error> {
        out.extend_from_slice(&self.call);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitFor {
    Submitted,
    InBlock,
    Finalized,
}

impl FromStr for WaitFor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "submitted" => Ok(WaitFor::Submitted),
            "in-block" => Ok(WaitFor::InBlock),
            "finalized" => Ok(WaitFor::Finalized),
            _ => Err(format!("Invalid wait-for value: {s}")),
        }
    }
}

pub async fn submit_call(
    chain_client: &OnlineClient<KiltConfig>,
    signer: &PairSigner<KiltConfig, Sr25519Pair>,
    call: &Vec<u8>,
    wait_for: WaitFor,
) -> Result<String, TxError> {
    let call = RawCall { call: call.clone() };
    let mut progress = chain_client
        .tx()
        .sign_and_submit_then_watch_default(&call, signer)
        .await?;
    log::info!(
        "Submitted Extrinsic with hash {:?}",
        progress.extrinsic_hash()
    );
    while let Some(Ok(status)) = progress.next_item().await {
        match status {
            subxt::tx::TxStatus::Future => {
                log::info!("Transaction is in the future queue");
            }
            subxt::tx::TxStatus::Ready => {
                log::info!("Extrinsic is ready");
            }
            subxt::tx::TxStatus::Broadcast(peers) => {
                log::info!("Extrinsic broadcasted to {:?}", peers);

                if wait_for == WaitFor::Submitted {
                    return Ok(format!("0x{}", hex::encode(progress.extrinsic_hash())));
                }
            }
            subxt::tx::TxStatus::InBlock(status) => {
                log::info!("Extrinsic included in block {:?}", status.block_hash());
                let events = status.fetch_events().await?;
                events.iter().for_each(|e| {
                    if let Ok(e) = e {
                        log::info!(
                            "{}.{}: {:#?}",
                            e.pallet_name(),
                            e.variant_name(),
                            e.event_metadata().pallet.docs()
                        );
                    }
                });
                if wait_for == WaitFor::InBlock {
                    return Ok(format!("0x{}", hex::encode(progress.extrinsic_hash())));
                }
            }
            subxt::tx::TxStatus::Retracted(hash) => {
                log::info!("Extrinsic retracted from block {:?}", hash);
            }
            subxt::tx::TxStatus::Finalized(status) => {
                log::info!("Extrinsic finalized in block {:?}", status.block_hash());
                if wait_for == WaitFor::Finalized {
                    return Ok(format!("0x{}", hex::encode(progress.extrinsic_hash())));
                }
            }
            subxt::tx::TxStatus::Usurped(hash) => {
                log::info!("Extrinsic usurped in block {:?}", hash);
            }
            subxt::tx::TxStatus::Dropped => {
                log::info!("Extrinsic dropped");
            }
            subxt::tx::TxStatus::Invalid => {
                log::info!("Extrinsic invalid");
            }
            subxt::tx::TxStatus::FinalityTimeout(hash) => {
                log::info!("Extrinsic finality timeout in block {:?}", hash);
            }
        }
    }
    Ok(format!("0x{}", hex::encode(progress.extrinsic_hash())))
}

pub async fn create_claim(
    claim_hash: sp_core::H256,
    ctype_hash: sp_core::H256,
    did_address: &AccountId32,
    chain_client: &OnlineClient<KiltConfig>,
    payer: &PairSigner<KiltConfig, Sr25519Pair>,
    signer: &PairSigner<KiltConfig, Sr25519Pair>,
) -> Result<Vec<u8>, subxt::Error> {
    let tx_counter = get_next_tx_counter(&chain_client, &did_address).await?;
    let block_number = get_current_block(&chain_client).await?;

    let call = RuntimeCall::Attestation(runtime_types::attestation::pallet::Call::add {
        claim_hash,
        ctype_hash,
        authorization: None,
    });

    let did_call = DidAuthorizedCallOperation {
        did: did_address.to_owned(),
        tx_counter,
        block_number,
        call,
        submitter: payer.account_id().to_owned().into(),
    };

    let encoded_call = did_call.encode();

    let signature = calculate_signature(&did_call.encode(), signer);
    let final_tx = runtime::tx().did().submit_did_call(did_call, signature);
    let events = chain_client
        .tx()
        .sign_and_submit_then_watch_default(&final_tx, payer)
        .await?
        .wait_for_finalized_success()
        .await;

    let created_event = events?.find_first::<runtime::attestation::events::AttestationCreated>()?;

    if let Some(_) = created_event {
        log::info!("Attestation with root hash {:?} created", claim_hash);
        Ok(encoded_call)
    } else {
        log::info!(
            "Attestation with root hash {:?} could not be created. Create Event not found",
            claim_hash
        );
        Err(subxt::Error::Other("Created Event not found".to_string()))
    }
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

pub async fn add_service_endpoint(
    did_address: &AccountId32,
    url: &str,
    service_id: &str,
    service_type: &str,
    submitter_signer: &PairSigner<KiltConfig, Pair>,
    chain_client: &OnlineClient<KiltConfig>,
) -> Result<(), subxt::Error> {
    let service_endpoint = DidEndpoint {
        id: BoundedVec(service_id.into()),
        service_types: BoundedVec(vec![BoundedVec(service_type.into())]),
        urls: BoundedVec(vec![BoundedVec(url.into())]),
    };

    let tx_counter = get_next_tx_counter(&chain_client, &did_address).await?;
    let block_number = get_current_block(&chain_client).await?;

    let call = RuntimeCall::Did(runtime_types::did::pallet::Call::add_service_endpoint {
        service_endpoint,
    });

    let did_call = DidAuthorizedCallOperation {
        did: did_address.to_owned(),
        tx_counter,
        call,
        block_number,
        submitter: submitter_signer.account_id().to_owned().into(),
    };

    let signature = calculate_signature(&did_call.encode(), submitter_signer);
    let final_tx = runtime::tx().did().submit_did_call(did_call, signature);
    let events = chain_client
        .tx()
        .sign_and_submit_then_watch_default(&final_tx, submitter_signer)
        .await?
        .wait_for_finalized_success()
        .await;

    let update_event = events?.find_first::<runtime::did::events::DidUpdated>()?;

    if let Some(_) = update_event {
        log::info!("Service endpoint with url: {:?} added", url);
        Ok(())
    } else {
        log::info!(
            "Service endpoint with url: {:?} could not be added. Update Event not found",
            url
        );
        Err(subxt::Error::Other("Update Event not found".to_string()))
    }
}

pub async fn remove_service_endpoint(
    did_address: &AccountId32,
    service_id: &str,
    submitter_signer: &PairSigner<KiltConfig, Pair>,
    chain_client: &OnlineClient<KiltConfig>,
) -> Result<(), subxt::Error> {
    let tx_counter = get_next_tx_counter(&chain_client, &did_address).await?;
    let block_number = get_current_block(&chain_client).await?;

    let call = RuntimeCall::Did(runtime_types::did::pallet::Call::remove_service_endpoint {
        service_id: BoundedVec(service_id.into()),
    });

    let did_call = DidAuthorizedCallOperation {
        did: did_address.to_owned(),
        tx_counter,
        call,
        block_number,
        submitter: submitter_signer.account_id().to_owned().into(),
    };

    let signature = calculate_signature(&did_call.encode(), submitter_signer);
    let final_tx = runtime::tx().did().submit_did_call(did_call, signature);
    let events = chain_client
        .tx()
        .sign_and_submit_then_watch_default(&final_tx, submitter_signer)
        .await?
        .wait_for_finalized_success()
        .await;

    let update_event = events?.find_first::<runtime::did::events::DidUpdated>()?;

    if let Some(_) = update_event {
        log::info!("Service endpoint with service id: {:?} removed", service_id);
        Ok(())
    } else {
        log::info!(
            "Service endpoint with service id: {:?} could not be added. Update Event not found",
            service_id
        );
        Err(subxt::Error::Other("Update Event not found".to_string()))
    }
}
