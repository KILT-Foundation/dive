use subxt::ext::sp_core::{sr25519, Pair};
use subxt::tx::PairSigner;

use super::file_manager::KeysFileStructure;
use crate::{device::error::DeviceError, kilt::KiltConfig};

pub trait KeyManager {
    fn get_payment_account_signer(&self) -> PairSigner<KiltConfig, sr25519::Pair>;
    fn get_did_auth_signer(&self) -> PairSigner<KiltConfig, sr25519::Pair>;
}

#[derive(Clone)]
pub struct PairKeyManager {
    payment_account_signer: sr25519::Pair,
    did_auth_signer: sr25519::Pair,
}

impl PairKeyManager {
    pub fn new(payment_random_seed: &str, auth_random_seed: &str) -> Result<Self, DeviceError> {
        let (payment_pair, _) = sr25519::Pair::from_phrase(payment_random_seed, None)?;
        let (did_auth_pair, _) = sr25519::Pair::from_phrase(auth_random_seed, None)?;
        Ok(Self {
            payment_account_signer: payment_pair,
            did_auth_signer: did_auth_pair,
        })
    }
}

impl KeyManager for PairKeyManager {
    fn get_payment_account_signer(&self) -> PairSigner<KiltConfig, sr25519::Pair> {
        PairSigner::new(self.payment_account_signer.clone())
    }

    fn get_did_auth_signer(&self) -> PairSigner<KiltConfig, sr25519::Pair> {
        PairSigner::new(self.did_auth_signer.clone())
    }
}

impl TryFrom<KeysFileStructure> for PairKeyManager {
    type Error = DeviceError;

    fn try_from(value: KeysFileStructure) -> Result<Self, Self::Error> {
        PairKeyManager::new(&value.payment_account_seed, &value.did_auth_seed)
    }
}
