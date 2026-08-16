use rusqlite::{Connection, Result};

pub fn open() -> Connection {
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "../data/idp.db".to_owned());
    let conn = Connection::open(db_path).expect("Failed to open DB");
    conn
}

pub fn run_migration() {}
