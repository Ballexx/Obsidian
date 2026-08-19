use rusqlite::{Connection, Result};

pub fn open() -> Connection {
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "../data/idp.db".to_owned());
    let conn = Connection::open(db_path).expect("Failed to open DB");
    conn
}

pub fn run_migration(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            email TEXT UNIQUE,
            email_verified INTEGER NOT NULL DEFAULT 0,
            password_hash TEXT NOT NULL,
            given_name TEXT,
            family_name TEXT,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS clients (
            client_id TEXT PRIMARY KEY,
            client_secret_hash TEXT,
            name TEXT NOT NULL,
            redirect_uris TEXT NOT NULL,
            grant_types TEXT NOT NULL DEFAULT 'authorization_code',
            scopes_allowed TEXT NOT NULL DEFAULT 'openid',
            is_confidential INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
    )
}
