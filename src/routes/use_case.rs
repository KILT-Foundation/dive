use actix_web::{post, web, HttpResponse, Responder, Scope};
use subxt::ext::sp_core::crypto::Ss58Codec;

use crate::{
    device::key_manager::KeyManager,
    error::ServerError,
    http_client::post_use_case_participation,
    kilt::did_helper::{get_did_service_endpoint, ADDRESS_FORMAT, DID_PREFIX},
    kilt::tx::{add_service_endpoint, remove_service_endpoint},
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

    // TODO: add these to app_state
    let use_case_service_endpoint_id = "#dena";
    let service_type = "KiltPublishedCredentialCollectionV1Type";
    let use_case_url = "http://localhost:8000";

    let maybe_service_endpoint = get_did_service_endpoint(
        &use_case_participation_message.use_case_did_url,
        use_case_service_endpoint_id,
        &app_state.chain_client,
    )
    .await?;

    let did = did_auth_signer
        .account_id()
        .to_ss58check_with_version(ADDRESS_FORMAT.into());

    match maybe_service_endpoint {
        None => {
            let _extrinsic_hash = remove_service_endpoint(
                &app_state.did_attester,
                use_case_service_endpoint_id,
                submitter_signer,
                &app_state.chain_client,
            )
            .await?;
        }
        Some(_) => {}
    };

    let formatted_did = format!("{}{}", DID_PREFIX, did);

    // Concatenate did urls - use case did url + device did url
    let concatenated_url = format!(
        "{}{}",
        use_case_participation_message.use_case_did_url, formatted_did
    );

    add_service_endpoint(
        &app_state.did_attester,
        &concatenated_url,
        use_case_service_endpoint_id,
        service_type,
        submitter_signer,
        &app_state.chain_client,
    )
    .await?;

    if use_case_participation_message.notify_use_case {
        post_use_case_participation(
            use_case_url,
            &use_case_participation_message.use_case_did_url,
        )
        .await?;
    }

    Ok(HttpResponse::Ok())
}

pub fn get_use_case_scope() -> Scope {
    web::scope("/api/v1/use-case")
}
