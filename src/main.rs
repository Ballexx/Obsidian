mod http;
mod oidc;
use http::server;
use std::sync::Arc;

pub mod log;

use crate::oidc::discovery::WellKnownConfig;

pub struct AppState {
    well_known_config: WellKnownConfig,
}

impl AppState {
    fn new() -> Self {
        AppState {
            well_known_config: WellKnownConfig::new(),
        }
    }
}

fn main() {
    let state = Arc::new(AppState {
        well_known_config: WellKnownConfig::new(),
    });

    server::listen("127.0.0.1", 3000, state);
}
