use std::path::{Path, PathBuf};

use walkdir::WalkDir;

const AUDIO_EXTENSIONS: &[&str] = &["flac", "mp3", "m4a", "aac", "ogg", "wav", "aiff", "aif"];

pub fn find_audio_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let dir = std::env::temp_dir().join(format!("spudbox_walk_test_{label}_{}_{nanos}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn touch(&self, relative: &str) {
            let full = self.0.join(relative);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(full, b"not real audio, just needs to exist").unwrap();
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn finds_known_audio_extensions_case_insensitively() {
        let dir = TempDir::new("extensions");
        dir.touch("song.flac");
        dir.touch("song.MP3");
        dir.touch("song.m4a");
        dir.touch("cover.jpg");
        dir.touch("notes.txt");

        let mut found: Vec<String> = find_audio_files(dir.path())
            .into_iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        found.sort();

        assert_eq!(found, vec!["song.MP3".to_string(), "song.flac".to_string(), "song.m4a".to_string()]);
    }

    #[test]
    fn recurses_into_subdirectories() {
        let dir = TempDir::new("recursion");
        dir.touch("Artist/Album/01 Track.flac");
        dir.touch("Artist/Album/cover.jpg");

        let found = find_audio_files(dir.path());
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("01 Track.flac"));
    }

    #[test]
    fn ignores_files_with_no_extension() {
        let dir = TempDir::new("no_extension");
        dir.touch("README");

        assert_eq!(find_audio_files(dir.path()), Vec::<PathBuf>::new());
    }
}
