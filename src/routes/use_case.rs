use actix_web::{get, post, web, HttpResponse, Responder, Scope};

use crate::{
    device::{file_manager::get_claim_content, key_manager::KeyManager},
    dto::UseCaseResponse,
    error::ServerError,
    http_client::post_use_case_participation,
    kilt::{
        did_helper::{get_did_address, get_did_service_endpoint},
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
    let did_auth_signer = keys.get_did_auth_signer().clone();
    let submitter_signer = keys.get_payment_account_signer();

    let use_case_service_endpoint_id = &app_state.use_case_service_endpoint_id;

    let UseCaseParticipationMessage {
        use_case_url,
        use_case_did_url,
        update_service_endpoint,
        notify_use_case,
        ..
    } = &use_case_participation_message.0;

    log::debug!("Use case participation posted: {:?}", use_case_did_url);

    let formatted_did = get_did_address(did_auth_signer.clone());

    // Concatenate did urls - use case did url + device did url
    let concatenated_url = format!("{}/{}", use_case_did_url, formatted_did);

    if *update_service_endpoint {
        if let Some(_) = get_did_service_endpoint(
            &formatted_did,
            use_case_service_endpoint_id,
            &app_state.chain_client,
        )
        .await?
        {
            remove_service_endpoint(
                use_case_service_endpoint_id,
                &submitter_signer,
                &did_auth_signer,
                &app_state.chain_client,
            )
            .await?;
        }

        add_service_endpoint(
            &concatenated_url,
            use_case_service_endpoint_id,
            &app_state.kilt_service_endpoint_type,
            &submitter_signer,
            &did_auth_signer,
            &app_state.chain_client,
        )
        .await?;
    }

    if *notify_use_case {
        let credential = get_claim_content()?;
        post_use_case_participation(use_case_url, &formatted_did, credential).await?;
    }

    Ok(HttpResponse::Ok().json(use_case_did_url))
}

#[get("")]
async fn get_use_case(app_state: web::Data<AppState>) -> Result<impl Responder, ServerError> {
    let keys = app_state.key_manager.lock().await;
    let did_auth_signer = keys.get_did_auth_signer().clone();
    let formatted_did = get_did_address(did_auth_signer);

    let use_case_service_endpoint_id = &app_state.use_case_service_endpoint_id;

    let maybe_service_endpoint = get_did_service_endpoint(
        &formatted_did,
        use_case_service_endpoint_id,
        &app_state.chain_client,
    )
    .await?;

    match maybe_service_endpoint {
        Some(service_endpoint) => {
            if let Ok(url) = String::from_utf8(service_endpoint.urls.0[0].0.clone()) {
                if let Some(use_case) = url.split("/").next() {
                    return Ok(HttpResponse::Ok().json(UseCaseResponse {
                        use_case: use_case.to_string(),
                    }));
                }
            }
            return Err(ServerError::UseCaseAPI(UseCaseAPIError::Format));
        }
        None => Err(ServerError::UseCaseAPI(UseCaseAPIError::NotFound)),
    }
}

pub fn get_use_case_scope() -> Scope {
    web::scope("/api/v1/use-case")
        .service(participate_to_use_case)
        .service(get_use_case)
}
