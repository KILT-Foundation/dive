use std::str::FromStr;

use actix_web::{HttpResponse, Responder, web};
use serde_json::json;
use subxt::{ext::{codec::{Encode, Decode}, sp_core::crypto::Ss58Codec, sp_runtime::MultiSignature}, tx::{Signer, TxPayload}, config::{polkadot::PolkadotExtrinsicParamsBuilder, substrate::Era}, OnlineClient};
use futures::StreamExt;

use crate::{
    kilt::{
        self,
        runtime_types::{
            did::did_details::{DidCreationDetails, DidSignature},
            sp_core::{bounded::bounded_btree_set::BoundedBTreeSet, ecdsa, ed25519, sr25519}, spiritnet_runtime::RuntimeCall,
        }, KiltConfig,
    },
    server::error::Error, crypto::manager::KeyManager,
};

use super::server::AppState;

pub async fn get_payment_account_address(app_state: web::Data<AppState>) -> Result<impl Responder, Error> {
    let mgr = app_state.key_manager.lock()?;
    let payment_account_id = mgr.get_payment_account_signer().account_id();
    let addr = payment_account_id.to_ss58check_with_version(38u16.into());
    Ok(HttpResponse::Ok().json(json!({"address": addr})))
}

pub async fn get_did(app_state: web::Data<AppState>) -> Result<impl Responder, Error> {
    let mgr = app_state.key_manager.lock()?;
    let did_auth_account_id = mgr.get_did_auth_signer().account_id();
    let addr = did_auth_account_id.to_ss58check_with_version(38u16.into());
    Ok(HttpResponse::Ok().json(json!({"did": format!("did:kilt:{}", addr)})))
}

pub async fn register_device_did(app_state: web::Data<AppState>) -> Result<impl Responder, Error> {
    let keys = crate::crypto::init_keys()?;
    let did_auth_signer = keys.get_did_auth_signer();
    let submitter_signer = keys.get_payment_account_signer();
    let details = DidCreationDetails {
        did: did_auth_signer.account_id().into(),
        submitter: submitter_signer.account_id().into(),
        new_key_agreement_keys: BoundedBTreeSet(vec![]),
        new_attestation_key: None,
        new_delegation_key: None,
        new_service_details: vec![],
    };
    let signature = did_auth_signer.sign(&details.encode());
    let did_sig = match signature {
        MultiSignature::Sr25519(sig) => DidSignature::Sr25519(sr25519::Signature(sig.0)),
        MultiSignature::Ed25519(sig) => DidSignature::Ed25519(ed25519::Signature(sig.0)),
        MultiSignature::Ecdsa(sig) => DidSignature::Ecdsa(ecdsa::Signature(sig.0)),
    };
    let tx = kilt::tx().did().create(details, did_sig);

    let api = app_state.kilt_api.lock()?;
    let signer = BoxSigner(submitter_signer);
    let events = api
        .tx()
        .sign_and_submit_then_watch_default(&tx, &signer)
        .await?
        .wait_for_finalized_success()
        .await?;
    println!("events: {:?}", events);
    Ok(HttpResponse::Ok().json(json!({"did": format!("did:kilt:{}", did_auth_signer.account_id().to_ss58check_with_version(38u16.into()))})))
}

const MAX_BODY_SIZE: usize = 262_144; // max payload size is 256k
pub async fn submit_extrinsic(app_state: web::Data<AppState>, mut payload: web::Payload)  -> Result<impl Responder, Error> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_BODY_SIZE {
            return Err(Error::Unknown);
        }
        body.extend_from_slice(&chunk);
    }

    let cli = app_state.kilt_api.lock()?;
    let keys = app_state.key_manager.lock()?;
    let signer = BoxSigner(keys.get_payment_account_signer());

    submit_call(&cli, &signer, &body.to_vec(), WaitFor::Finalized).await?;

    Ok(HttpResponse::Ok().json(json!({"did": "did:kilt:123"})))
}

struct BoxSigner(Box<dyn Signer<KiltConfig>>);

impl Signer<KiltConfig> for BoxSigner {
    fn account_id(&self) -> <KiltConfig as subxt::Config>::AccountId {
        self.0.account_id()
    }

    fn address(&self) -> <KiltConfig as subxt::Config>::Address {
        self.0.address()
    }

    fn sign(&self, data: &[u8]) -> MultiSignature {
        self.0.sign(data)
    }
}

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
enum WaitFor {
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

async fn submit_call(
    cli: &OnlineClient<KiltConfig>,
    signer: &BoxSigner,
    call: &Vec<u8>,
    wait_for: WaitFor,
) -> Result<(), Box<dyn std::error::Error>> {
    let call = RawCall { call: call.clone() };
    let mut progress = cli.tx().sign_and_submit_then_watch_default(&call, signer).await?;
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
                    return Ok(());
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
                    return Ok(());
                }
            }
            subxt::tx::TxStatus::Retracted(hash) => {
                log::info!("Extrinsic retracted from block {:?}", hash);
            }
            subxt::tx::TxStatus::Finalized(status) => {
                log::info!("Extrinsic finalized in block {:?}", status.block_hash());
                if wait_for == WaitFor::Finalized {
                    return Ok(());
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
    Ok(())
}