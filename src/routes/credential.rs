use actix_web::{get, web, HttpResponse, Responder, Scope};

use crate::{
    device::key_manager::KeyManager,
    error::ServerError,
    http_client::{check_jwt_health, get_credentials_from_attester, login_to_open_did},
    AppState,
};

#[get("")]
async fn get_credential(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let key_manager = app_state.key_manager.lock().await;
    let sign_pair = key_manager.get_did_auth_signer();
    let chain_client = &app_state.chain_client;

    let mut jwt_token = app_state.jwt_token.lock().await;

    let is_jwt_healty = check_jwt_health(&jwt_token);

    if !is_jwt_healty {
        let new_token = login_to_open_did(
            chain_client,
            sign_pair,
            &app_state.auth_client_id,
            &app_state.auth_endpoint,
            &app_state.redirect_url,
        )
        .await?;
        *jwt_token = new_token;
    }

    let data = get_credentials_from_attester(&jwt_token, &app_state.attester_endpoint).await?;

    Ok(HttpResponse::Ok().json(data))
}

pub fn get_credential_scope() -> Scope {
    web::scope("/api/v1/credential").service(get_credential)
}
