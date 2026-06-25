use rusqlite::{params, Connection};

use crate::error::AppError;

/// `None` deletes the row (unrated); `Some(r)` upserts it. Absence of a row
/// is the unrated sentinel — see migration 0003 for why this isn't a
/// nullable column instead.
pub fn set_rating(conn: &Connection, album_id: i64, rating: Option<f64>) -> Result<(), AppError> {
    match rating {
        Some(r) => conn.execute(
            "INSERT INTO album_ratings (album_id, rating) VALUES (?1, ?2)
             ON CONFLICT(album_id) DO UPDATE SET rating = excluded.rating",
            params![album_id, r],
        )?,
        None => conn.execute("DELETE FROM album_ratings WHERE album_id = ?1", params![album_id])?,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::queries::{albums, artists};
    use crate::db::schema::test_connection;

    fn setup_album(conn: &Connection) -> i64 {
        let artist_id = artists::upsert(conn, "Thrice").unwrap();
        albums::upsert(conn, "Vheissu", artist_id, Some(2005)).unwrap()
    }

    #[test]
    fn set_rating_then_list_all_reflects_it() {
        let conn = test_connection();
        let album_id = setup_album(&conn);
        set_rating(&conn, album_id, Some(8.5)).unwrap();
        let rows = albums::list_all(&conn, None).unwrap();
        assert_eq!(rows[0].rating, Some(8.5));
    }

    #[test]
    fn set_rating_overwrites_previous_value() {
        let conn = test_connection();
        let album_id = setup_album(&conn);
        set_rating(&conn, album_id, Some(3.0)).unwrap();
        set_rating(&conn, album_id, Some(9.5)).unwrap();
        let rows = albums::list_all(&conn, None).unwrap();
        assert_eq!(rows[0].rating, Some(9.5));
    }

    #[test]
    fn set_rating_none_clears_to_unrated() {
        let conn = test_connection();
        let album_id = setup_album(&conn);
        set_rating(&conn, album_id, Some(5.0)).unwrap();
        set_rating(&conn, album_id, None).unwrap();
        let rows = albums::list_all(&conn, None).unwrap();
        assert_eq!(rows[0].rating, None);
    }

    #[test]
    fn unrated_album_has_no_row_and_list_all_returns_none() {
        let conn = test_connection();
        setup_album(&conn);
        let rows = albums::list_all(&conn, None).unwrap();
        assert_eq!(rows[0].rating, None);
    }
}
