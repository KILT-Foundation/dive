#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use sp_core::blake2_256;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Debug)]
pub enum Error {
    ZkOpen,
    ZkClose,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ZkOpen => write!(f, "failed to open zymkey device"),
            Error::ZkClose => write!(f, "failed to close zymkey device"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub struct ZkCtx {
    ctx: zkCTX,
    is_closed: bool,
}

pub trait CryptoDevice {
    /// get_public_key returns the public key for the given slot
    fn get_public_key(&self, slot: u8) -> Result<Vec<u8>>;
    /// sign signs the given data with the given slot
    fn sign(&self, slot: u8, data: &[u8]) -> Result<Vec<u8>>;
    /// generate_key generates a new key and returns the slot
    fn generate_key(&mut self) -> Result<u8>;
    /// list returns a list of all slots
    fn list(&self) -> Result<Vec<u8>>;
    /// delete_key deletes the key in the given slot
    fn delete_key(&mut self, slot: u8) -> Result<()>;
    /// get_random_bytes returns a vector of random bytes
    fn get_random_bytes(&self, num_bytes: i32) -> Result<Vec<u8>>;
}

impl ZkCtx {
    // new initializes a new ZkCtx
    pub fn new() -> Result<Self> {
        let mut ctx: zkCTX = std::ptr::null_mut();
        let res = unsafe { zkOpen(&mut ctx) };
        if res != 0 {
            return Err(Error::ZkOpen);
        }
        Ok(Self {
            ctx,
            is_closed: false,
        })
    }

    // close closes the ZkCtx
    pub fn close(&mut self) -> Result<()> {
        if self.is_closed {
            return Ok(());
        }
        let res = unsafe { zkClose(self.ctx) };
        if res != 0 {
            return Err(Error::ZkClose);
        }
        self.is_closed = true;
        Ok(())
    }
}

impl Drop for ZkCtx {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl CryptoDevice for ZkCtx {
    fn get_public_key(&self, slot: u8) -> Result<Vec<u8>> {
        // define a u8 null pointer to receive the data
        let mut pub_key_bytes: *mut u8 = std::ptr::null_mut();
        let mut pub_key_len: i32 = 0;

        let res = unsafe {
            zkExportPubKey(
                self.ctx,
                &mut pub_key_bytes,
                &mut pub_key_len,
                slot as i32,
                false,
            )
        };
        if res != 0 {
            return Err(Error::ZkOpen);
        }

        let slice = unsafe { std::slice::from_raw_parts(pub_key_bytes, pub_key_len as usize) };
        let pub_key = slice.to_vec();
        unsafe { libc::free(pub_key_bytes as *mut std::ffi::c_void) };
        Ok(pub_key)
    }

    fn sign(&self, slot: u8, data: &[u8]) -> Result<Vec<u8>> {
        let hash = blake2_256(data);
        let mut sig_bytes: *mut u8 = std::ptr::null_mut();
        let mut sig_len: i32 = 0;
        let mut rec_id: u8 = 0;
        let res = unsafe {
            zkGenECDSASigFromDigestWithRecID(
                self.ctx,
                hash.as_ptr(),
                slot as i32,
                &mut sig_bytes,
                &mut sig_len,
                &mut rec_id,
            )
        };
        if res != 0 {
            return Err(Error::ZkOpen);
        }
        let slice = unsafe { std::slice::from_raw_parts(sig_bytes, sig_len as usize) };
        let mut sig = slice.to_vec();
        unsafe { libc::free(sig_bytes as *mut std::ffi::c_void) };
        sig.push(rec_id);
        Ok(sig)
    }

    fn generate_key(&mut self) -> Result<u8> {
        let slot = unsafe { zkGenKeyPair(self.ctx, ZK_EC_KEY_TYPE_ZK_SECP256K1) };
        if slot < 0 {
            return Err(Error::ZkOpen);
        }
        Ok(slot as u8)
    }

    fn list(&self) -> Result<Vec<u8>> {
        let mut slots: *mut i32 = std::ptr::null_mut();
        let mut max_slots_len: i32 = 32;
        let mut slots_len: i32 = 0;
        let res = unsafe {
            zkGetAllocSlotsList(
                self.ctx,
                false,
                &mut max_slots_len,
                &mut slots,
                &mut slots_len,
            )
        };
        if res != 0 {
            return Err(Error::ZkOpen);
        }
        let slice = unsafe { std::slice::from_raw_parts(slots, slots_len as usize) };
        let mut res = vec![];
        for i in 0..slots_len {
            res.push(slice[i as usize] as u8);
        }
        unsafe { libc::free(slots as *mut std::ffi::c_void) };
        Ok(res)
    }

    fn delete_key(&mut self, slot: u8) -> Result<()> {
        let res = unsafe { zkRemoveKey(self.ctx, slot as i32, false) };
        if res != 0 {
            return Err(Error::ZkOpen);
        }
        Ok(())
    }

    fn get_random_bytes(&self, num_bytes: i32) -> Result<Vec<u8>> {
        let mut rand_bytes: *mut u8 = std::ptr::null_mut();
        let res = unsafe { zkGetRandBytes(self.ctx, &mut rand_bytes, num_bytes) };
        if res != 0 {
            return Err(Error::ZkOpen);
        }
        let slice = unsafe { std::slice::from_raw_parts(rand_bytes, num_bytes as usize) };
        let result = slice.to_vec();
        unsafe { libc::free(rand_bytes as *mut std::ffi::c_void) };
        Ok(result)
    }
}

// struct HSM6Signer {
//     ctx: ZkCtx,
//     slot: u8,
// }

// impl HSM6Signer {
//     fn new(ctx: ZkCtx, slot: u8) -> Self {
//         Self { ctx, slot }
//     }
// }

// impl Signer<kilt::KiltConfig> for HSM6Signer {
//     fn account_id(&self) -> <kilt::KiltConfig as subxt::Config>::AccountId {
//         let pk = self.ctx.get_public_key(self.slot).unwrap();
//         AccountId32::new(blake2_256(&pk))
//     }

//     fn address(&self) -> <kilt::KiltConfig as subxt::Config>::Address {
//         let pk = self.ctx.get_public_key(self.slot).unwrap();
//         AccountId32::new(blake2_256(&pk)).into()
//     }

//     fn sign(&self, data: &[u8]) -> sp_runtime::MultiSignature {
//         let raw = self.ctx.sign(self.slot, data).unwrap();
//         let sig = MultiSignature::Ecdsa(sp_core::ecdsa::Signature::from_slice(&raw).unwrap());
//         sig
//     }
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>>{
//     nuke_all_slots()?;
//     let mut ctx = ZkCtx::new()?;
//     let slots = ctx.list()?;
//     println!("slots: {:?}", slots);
//     let payment_slot: u8;
//     let auth_slot:u8;
//     let attestation_slot: u8;
//     if slots.len() == 14 {
//         // no user slots were generated yet
//         payment_slot = ctx.generate_key()?;
//         auth_slot = ctx.generate_key()?;
//         attestation_slot = ctx.generate_key()?;
//         println!("payment_slot: {}", payment_slot);
//         println!("auth_slot: {}", auth_slot);
//         println!("attestation_slot: {}", attestation_slot);
//     } else if slots.len() == 17 {
//         payment_slot = slots[14];
//         auth_slot = slots[15];
//         attestation_slot = slots[16];
//     } else {
//         panic!("unexpected number of slots: {}", slots.len());
//     }

//     let slots = ctx.list()?;
//     println!("slots: {:?}", slots);
//     let payment_pk = ctx.get_public_key(payment_slot)?;
//     let auth_pk = ctx.get_public_key(auth_slot)?;
//     let attestation_pk = ctx.get_public_key(attestation_slot)?;
//     println!("payment_pk: {:?}", payment_pk);
//     println!("auth_pk: {:?}", auth_pk);
//     println!("attestation_pk: {:?}", attestation_pk);
//     let payment_account = AccountId32::new(blake2_256(&payment_pk));
//     let auth_account = AccountId32::new(blake2_256(&auth_pk));
//     let attestation_account = AccountId32::new(blake2_256(&attestation_pk));
//     let did = format!("did:kilt:{}", auth_account.to_ss58check_with_version(38u16.into()));
//     println!("did: {}", did);
//     println!("payment_account: {}", payment_account.to_ss58check_with_version(38u16.into()));
//     println!("attestation_account: {}", attestation_account.to_ss58check_with_version(38u16.into()));

//     let cli = kilt::connect("spiritnet").await?;
//     let receiver = {
//         let r: &[u8; 32] = payment_account.as_ref();
//         subxt::utils::AccountId32(r.to_owned())
//     };
//     let tx = kilt::tx().balances().transfer(receiver.into(), 1_000_000_000_000_000u128);

//     let events = cli
//         .tx()
//         .sign_and_submit_then_watch_default(&tx, &HSM6Signer::new(ctx, payment_slot))
//         .await?
//         .wait_for_finalized_success()
//         .await?;

//     // Find a Transfer event and print it.
//     let transfer_event = events.find_first::<kilt::balances::events::Transfer>()?;
//     if let Some(event) = transfer_event {
//         println!("Balance transfer success: {event:?}");
//     }
//     Ok(())
// }

// fn nuke_all_slots() -> Result<(), Box<dyn std::error::Error>> {
//     let mut ctx = ZkCtx::new()?;
//     let slots = ctx.list()?;
//     if slots.len() == 14 {
//         // no user slots were generated yet
//         return Ok(());
//     }
//     for slot in slots.iter().skip(14) {
//         println!("deleting slot: {}", slot);
//         ctx.delete_key(*slot)?;
//     }
//     ctx.close()?;
//     Ok(())
// }
