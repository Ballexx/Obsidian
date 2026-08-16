mod db;

mod http;
mod oidc;

use http::server;
use std::sync::{Arc, Mutex};

pub mod log;

use crate::oidc::discovery::WellKnownConfig;

pub struct AppState {
    well_known_config: WellKnownConfig,
    db: Arc<Mutex<Connection>>,
}

fn main() {
    run_migrations(&conn).expect("Failed to run migrations");
    let conn = Arc::new(Mutex::new(conn));

    let state = Arc::new(AppState {
        well_known_config: WellKnownConfig::new(),
        db: conn,
    });

    server::listen("127.0.0.1", 3000, state);
}
