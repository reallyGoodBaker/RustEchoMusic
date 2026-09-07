use std::path::Path;

use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::ItemKey;

use super::cache::LyricsCacheService;
use super::models::{LyricDocument, LyricLine};

pub struct LyricsService;

impl LyricsService {
    pub fn read_from_file(file_path: &Path, song_id: i64) -> Option<LyricDocument> {
        let tagged_file = Probe::open(file_path).ok()?.read().ok()?;
        let tag = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())?;

        let lyrics_text = tag
            .get_items(ItemKey::Lyrics)
            .next()
            .or_else(|| tag.get_items(ItemKey::UnsyncLyrics).next())
            .and_then(|item| item.value().text())?;

        let lines = Self::parse_lrc(lyrics_text);

        if lines.is_empty() {
            return None;
        }

        Some(LyricDocument { song_id, lines })
    }

    pub fn load_or_fetch(
        cache: &LyricsCacheService,
        song_id: i64,
        file_path: &Path,
    ) -> Option<LyricDocument> {
        if let Some(doc) = cache.load(song_id) {
            return Some(doc);
        }

        let doc = Self::read_from_file(file_path, song_id)?;

        if let Err(e) = cache.save(song_id, &doc) {
            eprintln!(
                "[Lyrics] Failed to cache lyrics for song {}: {}",
                song_id, e
            );
        }

        Some(doc)
    }

    pub fn parse_lrc(content: &str) -> Vec<LyricLine> {
        let mut lines = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(parsed) = Self::parse_lrc_line(trimmed) {
                lines.push(parsed);
            }
        }
        lines.sort_by(|a, b| a.timestamp_ms.cmp(&b.timestamp_ms));
        lines
    }

    fn parse_lrc_line(line: &str) -> Option<LyricLine> {
        if !line.starts_with('[') {
            return None;
        }
        let close = line.find(']')?;
        let tag = &line[1..close];
        let text = line[close + 1..].trim().to_string();

        let timestamp_ms = Self::parse_lrc_timestamp(tag)?;
        Some(LyricLine { timestamp_ms, text })
    }

    fn parse_lrc_timestamp(tag: &str) -> Option<u64> {
        let parts: Vec<&str> = tag.split(':').collect();

        let (hours, minutes, sec_str) = match parts.len() {
            2 => (0u64, parts[0].parse::<u64>().ok()?, parts[1]),
            3 => (
                parts[0].parse::<u64>().ok()?,
                parts[1].parse::<u64>().ok()?,
                parts[2],
            ),
            _ => return None,
        };

        let sec_parts: Vec<&str> = sec_str.split('.').collect();
        if sec_parts.len() != 2 {
            return None;
        }

        let seconds: u64 = sec_parts[0].parse().ok()?;
        let frac_str = sec_parts[1];
        let frac_value: u64 = frac_str.parse().ok()?;

        let milliseconds = match frac_str.len() {
            1 => frac_value * 100,
            2 => frac_value * 10,
            3 => frac_value,
            _ => frac_value,
        };

        Some(hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + milliseconds)
    }
}
