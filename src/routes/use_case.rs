use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use subxt::ext::sp_core::crypto::Ss58Codec;

use crate::{
    device::key_manager::KeyManager,
    dto::UseCaseResponse,
    error::ServerError,
    http_client::post_use_case_participation,
    kilt::{
        did_helper::{get_did_service_endpoint, ADDRESS_FORMAT, DID_PREFIX},
        error::UseCaseAPIError,
        tx::{add_service_endpoint, remove_service_endpoint},
    },
    routes::dto::*,
    AppState,
};

#[post("")]
async fn participate_to_use_case(
    app_state: web::Data<AppState>,
    use_case_participation_message: web::Json<UseCaseParticipationMessage>,
) -> Result<impl Responder, ServerError> {
    let keys = app_state.key_manager.lock().await;
    let did_auth_signer = &keys.get_did_auth_signer();
    let submitter_signer = &keys.get_payment_account_signer();

    log::debug!(
        "Use case participation posted: {:?}",
        use_case_participation_message.use_case_did_url
    );

    let use_case_service_endpoint_id = &app_state.use_case_service_endpoint_id;
    let use_case_url = use_case_participation_message.use_case_url.clone();

    let formatted_did = format!(
        "{}{}",
        DID_PREFIX,
        did_auth_signer
            .account_id()
            .to_ss58check_with_version(ADDRESS_FORMAT.into())
    );

    // Concatenate did urls - use case did url + device did url
    let concatenated_url = format!(
        "{}/{}",
        use_case_participation_message.use_case_did_url, formatted_did
    );

    if use_case_participation_message.update_service_endpoint {
        let maybe_service_endpoint = get_did_service_endpoint(
            &formatted_did,
            use_case_service_endpoint_id,
            &app_state.chain_client,
        )
        .await?;

        if maybe_service_endpoint.is_some() {
            remove_service_endpoint(
                use_case_service_endpoint_id,
                submitter_signer,
                did_auth_signer,
                &app_state.chain_client,
            )
            .await?;
        }

        add_service_endpoint(
            &concatenated_url,
            use_case_service_endpoint_id,
            &app_state.kilt_service_endpoint_type,
            submitter_signer,
            did_auth_signer,
            &app_state.chain_client,
        )
        .await?;
    }

    if use_case_participation_message.notify_use_case {
        post_use_case_participation(&use_case_url, &formatted_did).await?;
    }

    Ok(HttpResponse::Ok())
}

#[get("")]
async fn get_use_case(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let keys = app_state.key_manager.lock().await;
    let did_auth_signer = &keys.get_did_auth_signer();

    let formatted_did = format!(
        "{}{}",
        DID_PREFIX,
        did_auth_signer
            .account_id()
            .to_ss58check_with_version(ADDRESS_FORMAT.into())
    );

    let use_case_service_endpoint_id = &app_state.use_case_service_endpoint_id;

    let maybe_service_endpoint = get_did_service_endpoint(
        &formatted_did,
        use_case_service_endpoint_id,
        &app_state.chain_client,
    )
    .await?;

    match maybe_service_endpoint {
        Some(service_endpoint) => {
            let maybe_url = String::from_utf8(service_endpoint.urls.0[0].0.clone());
            match maybe_url {
                Ok(url) => {
                    let maybe_use_case = url.split("/").next();
                    match maybe_use_case {
                        Some(use_case) => Ok(HttpResponse::Ok().json(UseCaseResponse {
                            use_case: use_case.to_string(),
                        })),
                        None => Err(ServerError::UseCaseAPI(UseCaseAPIError::NotFound(
                            "Use case decoding error".to_string(),
                        ))),
                    }
                }
                Err(_) => Err(ServerError::UseCaseAPI(UseCaseAPIError::NotFound(
                    "Use case decoding error".to_string(),
                ))),
            }
        }
        None => Err(ServerError::UseCaseAPI(UseCaseAPIError::NotFound(
            "Use case not found".to_string(),
        ))),
    }
}

pub fn get_use_case_scope() -> Scope {
    web::scope("/api/v1/use-case")
        .service(participate_to_use_case)
        .service(get_use_case)
}
