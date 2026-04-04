use lofty::prelude::*;
use lofty::probe::Probe;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct MusicFile {
    pub path: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: u32,
}

impl MusicFile {
    pub fn new(path: &Path) -> Self {
        let file_path = path.to_string_lossy().to_string();
        let file_stem = path
            .file_stem()
            .unwrap_or_else(|| OsStr::new("Unknown"))
            .to_string_lossy()
            .to_string();

        let mut artist = "Unknown Artist".to_string();
        let mut album = "Unknown Album".to_string();
        let mut duration = 0;

        if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
            duration = tagged_file.properties().duration().as_secs() as u32;

            if let Some(tag) = tagged_file
                .primary_tag()
                .or_else(|| tagged_file.first_tag())
            {
                if let Some(a) = tag.artist() {
                    artist = a.to_string();
                }
                if let Some(al) = tag.album() {
                    album = al.to_string();
                }
            }
        }

        Self {
            path: file_path,
            name: file_stem,
            artist,
            album,
            duration,
        }
    }
}
