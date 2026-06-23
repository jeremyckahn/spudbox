use rusqlite::{OptionalExtension, Connection};

use crate::error::AppError;

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    conn.query_row("SELECT value FROM app_settings WHERE key = ?1", [key], |row| row.get(0))
        .optional()
        .map_err(AppError::from)
}

pub fn set(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::test_connection;

    #[test]
    fn get_returns_none_for_an_unset_key() {
        let conn = test_connection();
        assert_eq!(get(&conn, "volume").unwrap(), None);
    }

    #[test]
    fn set_then_get_round_trips() {
        let conn = test_connection();
        set(&conn, "volume", "0.75").unwrap();
        assert_eq!(get(&conn, "volume").unwrap(), Some("0.75".to_string()));
    }

    #[test]
    fn set_overwrites_the_previous_value() {
        let conn = test_connection();
        set(&conn, "volume", "0.75").unwrap();
        set(&conn, "volume", "0.5").unwrap();
        assert_eq!(get(&conn, "volume").unwrap(), Some("0.5".to_string()));
    }
}
