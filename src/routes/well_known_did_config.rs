use actix_web::{get, post, web, HttpResponse, Scope};

use crate::{
    error::ServerError,
    kilt::{
        error::CredentialAPIError,
        well_known_did_configuration::{create_well_known_did_config, WellKnownDidConfigData},
    },
    routes::dto::NewUrl,
    AppState,
};

#[post("")]
async fn update_well_known_did_origin(
    app_state: web::Data<AppState>,
    body: web::Json<NewUrl>,
) -> Result<HttpResponse, ServerError> {
    let mut old_well_known_did_config_data = app_state.well_known_did_config_data.lock().await;

    let new_well_known_did_config_data = WellKnownDidConfigData {
        did: old_well_known_did_config_data.did.clone(),
        key_uri: old_well_known_did_config_data.key_uri.clone(),
        origin: body.url.clone(),
        seed: old_well_known_did_config_data.seed.clone(),
    };

    *old_well_known_did_config_data = new_well_known_did_config_data;

    Ok(HttpResponse::Ok().json("Ok"))
}

#[get("")]
async fn well_known_did_config_handler(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, ServerError> {
    let well_known_did_config_data = app_state.well_known_did_config_data.lock().await.clone();

    let well_known_did_config = create_well_known_did_config(
        &well_known_did_config_data.did,
        &well_known_did_config_data.key_uri,
        &well_known_did_config_data.origin,
        &well_known_did_config_data.seed,
    )
    .map_err(|_| {
        ServerError::CredentialAPI(CredentialAPIError::Challenge(
            "Could not create well known did config",
        ))
    })?;

    Ok(HttpResponse::Ok().json(&well_known_did_config))
}

pub fn get_well_known_did_config_scope() -> Scope {
    web::scope("/.well-known/did-configuration.json")
        .service(update_well_known_did_origin)
        .service(well_known_did_config_handler)
}
