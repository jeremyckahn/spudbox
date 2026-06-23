use super::TrackInfo;

pub struct Queue {
    tracks: Vec<TrackInfo>,
    index: usize,
}

impl Queue {
    pub fn new(tracks: Vec<TrackInfo>, start_index: usize) -> Self {
        let index = start_index.min(tracks.len().saturating_sub(1));
        Self { tracks, index }
    }

    pub fn current(&self) -> Option<&TrackInfo> {
        self.tracks.get(self.index)
    }

    pub fn peek_next(&self) -> Option<&TrackInfo> {
        self.tracks.get(self.index + 1)
    }

    /// Moves to the next track, returning the new current track if any.
    pub fn advance(&mut self) -> Option<&TrackInfo> {
        self.index += 1;
        self.current()
    }

    pub fn move_to_previous(&mut self) -> Option<&TrackInfo> {
        if self.index == 0 {
            return None;
        }
        self.index -= 1;
        self.current()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn track_ids(&self) -> Vec<i64> {
        self.tracks.iter().map(|t| t.track_id).collect()
    }
}
