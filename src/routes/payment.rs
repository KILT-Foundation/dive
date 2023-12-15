use actix_web::{get, post, web, HttpResponse, Responder, Scope};

use crate::{
    device::key_manager::KeyManager,
    dto::{PayerAddress, TxResponse},
    error::ServerError,
    kilt::{
        error::{FormatError, TxError},
        tx::{submit_call, BoxSigner, WaitFor},
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
    let chain_client = &app_state.chain_client;
    let keys = app_state.key_manager.lock().await;
    let signer = BoxSigner(keys.get_payment_account_signer());
    let call_string = body.0;

    let trimmed_call = call_string.trim_start_matches("0x");

    let call = hex::decode(trimmed_call)
        .map_err(|e| ServerError::Tx(TxError::Format(FormatError::Hex(e))))?;

    let tx = submit_call(chain_client, &signer, &call, WaitFor::Finalized).await?;

    log::info!("Tx hash: {}", tx);
    Ok(HttpResponse::Ok().json(TxResponse { tx }))
}

pub fn get_payment_scope() -> Scope {
    web::scope("/api/v1/payment")
        .service(get_payment_account_address)
        .service(submit_extrinsic)
}
