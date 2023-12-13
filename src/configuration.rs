use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Parser)]
pub struct Configuration {
    #[clap(env)]
    pub wss_address: String,
    #[clap(env)]
    pub port: u16,
    #[clap(env)]
    pub front_end_path: String,
    #[clap(env)]
    pub attester_endpoint: String,
    #[clap(env)]
    pub auth_endpoint: String,
    #[clap(env)]
    pub auth_client_id: String,
    #[clap(env)]
    pub redirect_url: String,
}
