use actix_web::{get, web, HttpResponse, Scope};

use crate::{error::ServerError, AppState};

#[get("")]
async fn well_known_did_config_handler(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, ServerError> {
    Ok(HttpResponse::Ok().json(&app_state.well_known_did_config))
}

pub fn get_well_known_did_config_scope() -> Scope {
    web::scope("/.well-known/did-configuration.json").service(well_known_did_config_handler)
}
