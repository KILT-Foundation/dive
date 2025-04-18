use actix_web::{get, post, web, HttpResponse, Responder, Scope};

use crate::{
    device::key_manager::KeyManager,
    dto::PayerAddress,
    error::ServerError,
    kilt::{
        connect,
        error::{FormatError, TxError},
        tx::{submit_call, WaitFor},
    },
    AppState,
};

#[get("")]
async fn get_payment_account_address(
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ServerError> {
    let address = app_state.payment_addr.clone();
    Ok(HttpResponse::Ok().json(PayerAddress { address }))
}

#[post("")]
async fn submit_extrinsic(
    app_state: web::Data<AppState>,
    body: web::Json<String>,
) -> Result<impl Responder, ServerError> {
    let chain_client = connect(&app_state.wss_endpoint).await?;
    let keys = app_state.key_manager.lock().await;
    let signer = keys.get_payment_account_signer();
    let call_string = body.0;

    let trimmed_call = call_string.trim_start_matches("0x");

    let call = hex::decode(trimmed_call)
        .map_err(|e| ServerError::Tx(TxError::Format(FormatError::Hex(e))))?;

    let tx = submit_call(&chain_client, &signer, call, WaitFor::Finalized).await?;

    log::info!("Tx hash: {}", tx);
    Ok(HttpResponse::Ok())
}

pub fn get_payment_scope() -> Scope {
    web::scope("/api/v1/payment")
        .service(get_payment_account_address)
        .service(submit_extrinsic)
}
