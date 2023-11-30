#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use sp_core::blake2_256;

use super::error::DeviceError;
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(not(feature = "hsm6"))]
use rand::Rng;

#[cfg(feature = "hsm6")]
pub fn get_random_bytes(
    num_bytes: i32,
) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
    let hsm6 = device::ZkCtx::new()?;
    let random_bytes = hsm6.get_random_bytes(num_bytes)?;
    Ok(random_bytes)
}

#[cfg(not(feature = "hsm6"))]
pub fn get_random_bytes(_num_bytes: i32) -> std::result::Result<Vec<u8>, DeviceError> {
    let mut random_bytes = vec![0u8; 32];
    rand::thread_rng().fill(&mut random_bytes[..]);
    Ok(random_bytes)
}

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
