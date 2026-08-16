use crate::AppState;
use crate::{http::response::Response, http::status::StatusCode};
use serde_json;
use std::{env, format, vec};

#[derive(serde::Serialize)]
pub struct WellKnownConfig {
    issuer: String,
    auth_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
    jwks_uri: String,
    response_types_supported: Vec<String>,
    subject_types_supported: Vec<String>,
    id_token_signing_alg_values_supported: Vec<String>,
    scopes_supported: Vec<String>,
}

impl WellKnownConfig {
    pub fn new() -> Self {
        let issuer =
            std::env::var("ISSUER_URL").unwrap_or_else(|_| "http://localhost:3000".to_owned());

        WellKnownConfig {
            issuer: issuer.clone(),
            auth_endpoint: format!("{}/authorize", issuer),
            token_endpoint: format!("{}/token", issuer),
            userinfo_endpoint: format!("{}/userinfo", issuer),
            jwks_uri: format!("{}/.well-known/jwks.json", issuer),
            response_types_supported: vec!["code".to_owned()],
            subject_types_supported: vec!["public".to_owned()],
            id_token_signing_alg_values_supported: vec!["RS256".to_owned()],
            scopes_supported: vec![
                "openid".to_owned(),
                "profile".to_owned(),
                "email".to_owned(),
                "offline_access".to_owned(),
            ],
        }
    }

    pub fn handle_discovery(state: &AppState) -> Response {
        let config = &state.well_known_config;

        let mut response = Response::new();
        response.set_response_header("Content-Type".to_owned(), "application/json".to_owned());

        let Ok(json_body) = serde_json::to_string(&state.well_known_config) else {
            response.set_status(StatusCode::InternalServerError);
            return response;
        };

        response.set_response_body(json_body);
        response
    }
}
