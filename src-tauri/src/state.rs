use r2d2_sqlite::SqliteConnectionManager;

use crate::audio::PlayerHandle;

pub type DbPool = r2d2::Pool<SqliteConnectionManager>;

pub struct AppState {
    pub db: DbPool,
    pub player: PlayerHandle,
}
