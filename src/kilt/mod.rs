pub mod did_helper;
pub mod error;
pub mod tx;

use subxt::ext::sp_runtime::traits::{IdentifyAccount, Verify};
use subxt::{config::polkadot::PolkadotExtrinsicParams, config::Config, OnlineClient};

#[cfg(feature = "spiritnet")]
#[subxt::subxt(runtime_metadata_path = "metadata_spiritnet_11110.scale")]
pub mod runtime {}

#[cfg(not(feature = "spiritnet"))]
#[subxt::subxt(runtime_metadata_path = "./metadata_peregrine_11110.scale")]
pub mod runtime {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KiltConfig;
impl Config for KiltConfig {
    type Hash = subxt::ext::sp_core::H256;
    type Hasher = <subxt::config::SubstrateConfig as Config>::Hasher;
    type AccountId = <<Self::Signature as Verify>::Signer as IdentifyAccount>::AccountId;
    type Address = subxt::ext::sp_runtime::MultiAddress<Self::AccountId, ()>;
    type Header = subxt::config::substrate::SubstrateHeader<u64, Self::Hasher>;
    type Signature = subxt::ext::sp_runtime::MultiSignature;
    type ExtrinsicParams = PolkadotExtrinsicParams<Self>;
}
