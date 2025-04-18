use actix_web::{get, post, web, HttpResponse, Responder, Scope};

use crate::{
    device::{
        file_manager::{get_claim_content, save_claim_content},
        key_manager::KeyManager,
    },
    dto::Credential,
    error::ServerError,
    http_client::{check_jwt_health, login_to_open_did, post_claim_to_attester},
    kilt::connect,
    AppState,
};

#[get("")]
async fn get_base_claim() -> Result<impl Responder, ServerError> {
    let claim = get_claim_content()?;
    Ok(HttpResponse::Ok().json(claim))
}

#[post("")]
async fn post_base_claim(
    body: web::Json<Credential>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ServerError> {
    let base_claim = body.0;

    log::debug!("Base claim posted: {:?}", base_claim);

    let key_manager = app_state.key_manager.lock().await;

    let sign_pair = key_manager.get_did_auth_signer();
    let chain_client = connect(&app_state.wss_endpoint).await?;

    let mut jwt_token = app_state.jwt_token.lock().await;

    let is_jwt_healthy = check_jwt_health(&jwt_token);

    if !is_jwt_healthy {
        let new_token = login_to_open_did(
            &chain_client,
            sign_pair,
            &app_state.auth_client_id,
            &app_state.auth_endpoint,
            &app_state.redirect_url,
        )
        .await?;
        *jwt_token = new_token;
    }

    post_claim_to_attester(&jwt_token, &base_claim, &app_state.attester_endpoint).await?;

    save_claim_content(&base_claim)?;

    Ok(HttpResponse::Ok().json(base_claim))
}

pub fn get_claim_scope() -> Scope {
    web::scope("/api/v1/claim")
        .service(get_base_claim)
        .service(post_base_claim)
}
