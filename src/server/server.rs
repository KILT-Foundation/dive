use std::sync::{Arc, Mutex};

use actix_files as fs;
use actix_web::{web, App, HttpServer};
use subxt::OnlineClient;

use crate::{
    crypto::manager::PairKeyManager,
    kilt::{self, KiltConfig},
    server::{routes::get_payment_account_address, Error},
};

use super::routes::submit_extrinsic;

pub struct Server {
    port: u16,
    source_dir: String,
}

#[derive(Clone)]
pub struct AppState {
    pub key_manager: Arc<Mutex<PairKeyManager>>,
    pub kilt_api: Arc<Mutex<OnlineClient<KiltConfig>>>,
}

impl Server {
    pub fn new(port: u16, source_dir: &str) -> Self {
        Self {
            port,
            source_dir: source_dir.to_string(),
        }
    }

    pub async fn run(&self) -> Result<(), Error> {
        let source_dir = self.source_dir.clone();
        let api = kilt::connect("wss://spiritnet.kilt.io:443").await?;
        let app_state = AppState {
            key_manager: Arc::new(Mutex::new(crate::crypto::init_keys()?)),
            kilt_api: Arc::new(Mutex::new(api)),
        };
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .service(
                    web::scope("/api/v1")
                        .route("/payment", web::get().to(get_payment_account_address))
                        .route("/payment", web::post().to(submit_extrinsic))
                        .route("/did", web::get().to(crate::server::routes::get_did))
                        .route(
                            "/did",
                            web::post().to(crate::server::routes::register_device_did),
                        )
                        .route(
                            "/claim",
                            web::post().to(crate::server::routes::post_base_claim),
                        )
                        .route(
                            "/claim",
                            web::get().to(crate::server::routes::get_base_claim),
                        ),
                )
                .service(fs::Files::new("/", &source_dir).index_file("index.html"))
        })
        .bind(("0.0.0.0", self.port))?
        .run()
        .await?;
        Ok(())
    }
}
