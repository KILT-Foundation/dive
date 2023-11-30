mod configuration;

use clap::Parser;
use dive::{crypto::init_keys, server};

use crate::configuration::Configuration;

#[tokio::main]
async fn main() -> Result<(), server::Error> {
    // setup logger
    env_logger::init();

    // Create config file from env
    let config = Configuration::parse();

    let source_dir = config.front_end_path;
    let wss_endpoint = config.wss_address;
    let port = config.port;
    let auth_endpoint = config.auth_endpoint;
    let attester_endpoint = config.attester_endpoint;
    let auth_client_id = config.auth_client_id;

    let key_manager = init_keys()?;

    server::run(
        source_dir,
        wss_endpoint,
        port,
        key_manager,
        auth_endpoint,
        attester_endpoint,
        auth_client_id,
    )
    .await
}
