use lofty::picture::PictureType;
use lofty::prelude::*;
use lofty::probe::Probe;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicFile {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: u32,
    pub duration_formatted: String,
    pub sample_rate: u32,
    pub bit_rate: u32,
    pub channels: u8,
    pub track_number: Option<u32>,
    pub cover_base64: String,
}

impl MusicFile {
    pub fn new(path: &Path) -> Self {
        let file_path = path.to_string_lossy().to_string();
        let file_stem = path
            .file_stem()
            .unwrap_or_else(|| OsStr::new("Unknown"))
            .to_string_lossy()
            .to_string();

        let mut title = file_stem.clone();
        let mut artist = "Unknown Artist".to_string();
        let mut album = "Unknown Album".to_string();
        let mut duration_secs: u32 = 0;
        let mut sample_rate: u32 = 0;
        let mut bit_rate: u32 = 0;
        let mut channels: u8 = 0;
        let mut track_number: Option<u32> = None;
        let mut cover_base64 = String::new();

        if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
            let props = tagged_file.properties();

            duration_secs = props.duration().as_secs() as u32;
            sample_rate = props.sample_rate().unwrap_or(0);
            channels = props.channels().unwrap_or(0);

            bit_rate = props
                .overall_bitrate()
                .or(props.audio_bitrate())
                .unwrap_or(0);

            if let Some(tag) = tagged_file
                .primary_tag()
                .or_else(|| tagged_file.first_tag())
            {
                if let Some(t) = tag.title() {
                    title = t.to_string();
                }
                if let Some(a) = tag.artist() {
                    artist = a.to_string();
                }
                if let Some(al) = tag.album() {
                    album = al.to_string();
                }
                track_number = tag.track();

                if let Some(picture) = tag.pictures().iter().find(|pic| {
                    pic.pic_type() == PictureType::CoverFront
                        || pic.pic_type() == PictureType::CoverBack
                }) {
                    let mime_str = picture
                        .mime_type()
                        .map(|m| m.as_str())
                        .unwrap_or("image/jpeg");
                    let data = picture.data();

                    use base64::engine::{general_purpose, Engine as _};
                    cover_base64 = format!(
                        "data:{};base64,{}",
                        mime_str,
                        general_purpose::STANDARD.encode(data)
                    );
                }
            }
        }

        let minutes = duration_secs / 60;
        let seconds = duration_secs % 60;
        let duration_formatted = format!("{:02}:{:02}", minutes, seconds);

        Self {
            path: file_path,
            title,
            artist,
            album,
            duration_secs,
            duration_formatted,
            sample_rate,
            bit_rate,
            channels,
            track_number,
            cover_base64,
        }
    }

    pub fn display_duration(&self) -> &str {
        &self.duration_formatted
    }
}
