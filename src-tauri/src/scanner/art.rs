use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::GenericImageView;
use lofty::prelude::*;
use serde::Serialize;

use crate::db::queries::{albums, tracks};
use crate::error::AppError;
use crate::state::DbPool;

const THUMBNAIL_MAX_DIM: u32 = 480;
const FOLDER_ART_NAMES: &[&str] = &[
    "cover.jpg",
    "cover.png",
    "folder.jpg",
    "folder.png",
    "front.jpg",
    "front.png",
];

#[derive(Debug, Default, Clone, Serialize)]
pub struct ArtStats {
    pub embedded: usize,
    pub folder: usize,
    pub none: usize,
    pub errors: usize,
}

/// Extracts and caches one thumbnail per album lacking art: embedded
/// picture (via lofty, re-reading a single sample track) first, then a
/// folder cover image, run as a separate pass after the main tag scan
/// rather than during it (re-reading every track just for its picture
/// during the parallel tag pass would mean holding many large embedded
/// images in memory at once for art that's identical across an album).
pub fn backfill_album_art(pool: &DbPool, cache_dir: &Path) -> Result<ArtStats, AppError> {
    let conn = pool.get()?;
    let album_ids = albums::list_missing_art(&conn)?;
    let mut stats = ArtStats::default();

    if album_ids.is_empty() {
        return Ok(stats);
    }
    std::fs::create_dir_all(cache_dir)?;

    for album_id in album_ids {
        let Some(sample_path) = tracks::sample_path_for_album(&conn, album_id)? else {
            albums::set_art(&conn, album_id, None, "none")?;
            stats.none += 1;
            continue;
        };
        let sample_path = PathBuf::from(sample_path);

        let (bytes, source) = match embedded_picture_bytes(&sample_path) {
            Some(b) => (b, "embedded"),
            None => match folder_art_bytes(&sample_path) {
                Some(b) => (b, "folder"),
                None => {
                    albums::set_art(&conn, album_id, None, "none")?;
                    stats.none += 1;
                    continue;
                }
            },
        };

        let dest = cache_dir.join(format!("{album_id}.jpg"));
        match resize_and_save(&bytes, &dest) {
            Ok(()) => {
                albums::set_art(&conn, album_id, Some(&dest.to_string_lossy()), source)?;
                if source == "embedded" {
                    stats.embedded += 1;
                } else {
                    stats.folder += 1;
                }
            }
            Err(_) => {
                albums::set_art(&conn, album_id, None, "none")?;
                stats.errors += 1;
            }
        }
    }

    Ok(stats)
}

fn embedded_picture_bytes(path: &Path) -> Option<Vec<u8>> {
    let tagged_file = lofty::read_from_path(path).ok()?;
    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag())?;
    tag.pictures().first().map(|p| p.data().to_vec())
}

fn folder_art_bytes(track_path: &Path) -> Option<Vec<u8>> {
    let folder = track_path.parent()?;
    for name in FOLDER_ART_NAMES {
        let candidate = folder.join(name);
        if candidate.is_file() {
            if let Ok(bytes) = std::fs::read(&candidate) {
                return Some(bytes);
            }
        }
    }
    None
}

fn resize_and_save(bytes: &[u8], dest: &Path) -> Result<(), image::ImageError> {
    let img = image::load_from_memory(bytes)?;
    let (w, h) = img.dimensions();
    let longest = w.max(h);
    let resized = if longest > THUMBNAIL_MAX_DIM {
        let scale = THUMBNAIL_MAX_DIM as f32 / longest as f32;
        img.resize(
            (w as f32 * scale).round() as u32,
            (h as f32 * scale).round() as u32,
            FilterType::Lanczos3,
        )
    } else {
        img
    };
    resized.to_rgb8().save_with_format(dest, image::ImageFormat::Jpeg)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let dir = std::env::temp_dir().join(format!("spudbox_art_test_{label}_{}_{nanos}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn solid_png_bytes(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbImage::new(width, height);
        let dynamic = image::DynamicImage::ImageRgb8(img);
        let mut bytes = Vec::new();
        dynamic.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png).unwrap();
        bytes
    }

    #[test]
    fn resize_and_save_shrinks_images_above_the_thumbnail_max() {
        let dir = TempDir::new("shrink");
        let dest = dir.0.join("out.jpg");
        let bytes = solid_png_bytes(1000, 500);

        resize_and_save(&bytes, &dest).unwrap();

        let saved = image::open(&dest).unwrap();
        assert_eq!(saved.width(), THUMBNAIL_MAX_DIM);
        assert_eq!(saved.height(), 240, "aspect ratio (2:1) should be preserved when scaling down");
    }

    #[test]
    fn resize_and_save_leaves_small_images_unscaled() {
        let dir = TempDir::new("noscale");
        let dest = dir.0.join("out.jpg");
        let bytes = solid_png_bytes(100, 50);

        resize_and_save(&bytes, &dest).unwrap();

        let saved = image::open(&dest).unwrap();
        assert_eq!((saved.width(), saved.height()), (100, 50));
    }

    #[test]
    fn resize_and_save_rejects_invalid_image_bytes() {
        let dir = TempDir::new("invalid");
        let dest = dir.0.join("out.jpg");
        assert!(resize_and_save(b"not an image", &dest).is_err());
    }

    #[test]
    fn folder_art_bytes_finds_a_known_cover_filename() {
        let dir = TempDir::new("folder_art");
        let png = solid_png_bytes(10, 10);
        std::fs::write(dir.0.join("cover.jpg"), &png).unwrap();
        let track_path = dir.0.join("01 Track.flac");
        std::fs::write(&track_path, b"not real audio").unwrap();

        assert_eq!(folder_art_bytes(&track_path), Some(png));
    }

    #[test]
    fn folder_art_bytes_is_none_when_no_cover_image_present() {
        let dir = TempDir::new("no_folder_art");
        let track_path = dir.0.join("01 Track.flac");
        std::fs::write(&track_path, b"not real audio").unwrap();

        assert_eq!(folder_art_bytes(&track_path), None);
    }
}
