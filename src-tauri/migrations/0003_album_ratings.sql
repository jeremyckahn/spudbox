-- Absence of a row means "unrated". Deliberately a separate table rather
-- than a nullable column alongside other stats (unlike track_stats.rating,
-- which lives in a multi-purpose row with play_count/last_played/etc.) —
-- this table has exactly one purpose.
CREATE TABLE album_ratings (
    album_id INTEGER PRIMARY KEY REFERENCES albums(id) ON DELETE CASCADE,
    rating   REAL NOT NULL
);
