use dive::crypto::manager::KeyManager;

use subxt::ext::sp_core::crypto::Ss58Codec;

use dive::crypto::init_keys;
use dive::{kilt, server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _api = kilt::connect("wss://spiritnet.kilt.io:443").await?;
    println!("connected to spiritnet");
    let keys = init_keys()?;
    let payment_account_id = keys.get_payment_account_signer().account_id();
    let did_auth_account_id = keys.get_did_auth_signer().account_id();
    println!(
        "payment_account_id: {}",
        payment_account_id.to_ss58check_with_version(38u16.into())
    );
    println!(
        "DID: did:kilt:{}",
        did_auth_account_id.to_ss58check_with_version(38u16.into())
    );
    let web_port: u16 = 1606;
    let web_root = "./web";
    println!("starting server on port {}", web_port);
    let server = server::Server::new(web_port, web_root);
    server.run().await?;
    Ok(())
}
