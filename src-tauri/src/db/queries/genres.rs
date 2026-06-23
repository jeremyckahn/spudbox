use rusqlite::Connection;

use crate::error::AppError;

pub fn upsert(conn: &Connection, name: &str) -> Result<i64, AppError> {
    let id: i64 = conn.query_row(
        "INSERT INTO genres (name) VALUES (?1)
         ON CONFLICT(name) DO UPDATE SET name = excluded.name
         RETURNING id",
        [name],
        |row| row.get(0),
    )?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::test_connection;

    #[test]
    fn upsert_is_idempotent_for_the_same_name() {
        let conn = test_connection();
        let first = upsert(&conn, "Post-Hardcore").unwrap();
        let second = upsert(&conn, "Post-Hardcore").unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn upsert_gives_different_names_different_ids() {
        let conn = test_connection();
        let a = upsert(&conn, "Post-Hardcore").unwrap();
        let b = upsert(&conn, "Mathcore").unwrap();
        assert_ne!(a, b);
    }
}
