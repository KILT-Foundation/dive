use crate::{configuration::Configuration, utils::parse_configuration_to_app_state, AppState};

pub async fn create_mocked_app_state() -> AppState {
    let config = Configuration::get_test_config();

    let app_state = parse_configuration_to_app_state(config)
        .await
        .expect("Creating app state should not fail");

    app_state
}
