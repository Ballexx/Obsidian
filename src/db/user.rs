use rusqlite::{Connection, OptionalExtension, Result};

struct User {
    id: String,
    username: String,
    email: String,
    password_hash: String,
}

impl User {
    pub fn create_user(conn: &Connection) {}

    pub fn get_user_by_username(conn: &Connection, username: String) {
        let result = conn
            .query_row(
                "SELECT id, username, email, password_hash FROM users WHERE username = ?1",
                [username],
                |row| {
                    Ok(User {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        email: row.get(2)?,
                        password_hash: row.get(3)?,
                    })
                },
            )
            .optional();
    }
}
