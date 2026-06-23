use rusqlite::{params, Connection};

use crate::error::AppError;

/// Records a play: an append-only history row plus a denormalized rollup
/// (play_count/last_played) for fast "most played"/"recently played"
/// sorting later without aggregating history at query time. Counted when a
/// track starts, not on completion — simpler and avoids needing the engine
/// to reliably distinguish "played to the end" from "skipped," which the
/// data so far doesn't need.
pub fn record_play(conn: &Connection, track_id: i64, played_at: i64) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO play_history (track_id, played_at, completed) VALUES (?1, ?2, 0)",
        params![track_id, played_at],
    )?;
    conn.execute(
        "INSERT INTO track_stats (track_id, play_count, last_played, rating, is_favorite)
         VALUES (?1, 1, ?2, NULL, 0)
         ON CONFLICT(track_id) DO UPDATE SET
             play_count = play_count + 1,
             last_played = excluded.last_played",
        params![track_id, played_at],
    )?;
    Ok(())
}
