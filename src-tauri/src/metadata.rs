use std::fs;
use std::path::Path;

use lofty::prelude::*;
use lofty::probe::Probe;

use crate::models::NewTrack;

pub fn parse_single_track(file_path: &Path) -> Option<NewTrack> {
    let tagged_file = Probe::open(file_path).ok()?.read().ok()?;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let title = tag
        .and_then(|t| t.title())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown Track")
                .to_string()
        });

    let artist = tag
        .and_then(|t| t.artist())
        .map(|s| s.to_string())
        .or_else(|| Some("Unknown Artist".to_string()));

    let album = tag
        .and_then(|t| t.album())
        .map(|s| s.to_string())
        .or_else(|| Some("Unknown Album".to_string()));

    let props = tagged_file.properties();
    let duration = (props.duration().as_secs_f64() * 1000.0).round() as i64;
    let path = file_path.to_string_lossy().into_owned();
    let file_size = fs::metadata(file_path)
        .ok()
        .and_then(|metadata| i64::try_from(metadata.len()).ok());

    Some(NewTrack {
        title,
        artist,
        album,
        duration,
        path,
        cover: None,
        file_size,
    })
}
