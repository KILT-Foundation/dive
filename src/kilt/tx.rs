use std::str::FromStr;
use subxt::{
    ext::sp_runtime::MultiSignature,
    tx::{Signer, TxPayload},
    OnlineClient,
};

use super::{error::TxError, KiltConfig};

pub struct BoxSigner(pub Box<dyn Signer<KiltConfig>>);

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
    cli: &OnlineClient<KiltConfig>,
    signer: &BoxSigner,
    call: &Vec<u8>,
    wait_for: WaitFor,
) -> Result<String, TxError> {
    let call = RawCall { call: call.clone() };
    let mut progress = cli
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
