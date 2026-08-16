use crate::log::{LogEntry, LogLevel};
use std::{env, format, vec};

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

    fn handle_discovery() {}
}
