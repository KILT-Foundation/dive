use actix_web::{test, web, App};

use crate::{
    dto::DidAddress,
    kilt::did_helper::DID_PREFIX,
    routes::{
        challenge::{get_challenge_scope, ChallengeData},
        did::get_did_scope,
    },
    test_utils::create_mocked_app_state,
};

#[actix_rt::test]
async fn test_challenge_handler() {
    let endpoint = "/api/v1/challenge";
    let app_state = create_mocked_app_state().await;
    // Arrange
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(get_challenge_scope()),
    )
    .await;

    // Act
    let request = test::TestRequest::get().uri(endpoint).to_request();
    let response = test::call_service(&mut app, request).await;

    // Assert
    assert!(response.status().is_success());

    let challenge_data: ChallengeData = test::read_body_json(response).await;
    assert_eq!(challenge_data.app_name, app_state.app_name);
    assert_eq!(
        challenge_data.encryption_key_uri,
        app_state.session_encryption_public_key_uri
    );
    assert_eq!(challenge_data.challenge.len(), 16);
}

#[actix_rt::test]
async fn test_get_did_success() {
    // Arrange
    let endpoint = "/api/v1/did";
    let app_state = create_mocked_app_state().await;
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(get_did_scope()),
    )
    .await;

    // Act
    let request = test::TestRequest::get().uri(endpoint).to_request();
    let response = test::call_service(&mut app, request).await;

    // Assert
    assert!(response.status().is_success());

    let did_response: DidAddress = test::read_body_json(response).await;

    assert_eq!(
        did_response.did,
        format!("{}{}", DID_PREFIX, app_state.did_addr)
    );
}
