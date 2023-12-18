mod claim;
mod credential;
mod did;
mod payment;

pub use claim::get_claim_scope;
pub use credential::get_credential_scope;
pub use did::get_did_scope;
pub use payment::get_payment_scope;
