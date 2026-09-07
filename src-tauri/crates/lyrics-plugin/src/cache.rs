use std::path::PathBuf;

use super::models::LyricDocument;

pub struct LyricsCacheService {
    cache_dir: PathBuf,
}

impl LyricsCacheService {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    pub fn init(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(|e| format!("Failed to create lyrics cache dir: {}", e))
    }

    fn cache_path(&self, song_id: i64) -> PathBuf {
        self.cache_dir.join(format!("{}.json", song_id))
    }

    pub fn save(&self, song_id: i64, document: &LyricDocument) -> Result<(), String> {
        let content = serde_json::to_string_pretty(document)
            .map_err(|e| format!("Failed to serialize lyrics: {}", e))?;
        std::fs::write(self.cache_path(song_id), content)
            .map_err(|e| format!("Failed to write lyrics cache: {}", e))
    }

    pub fn load(&self, song_id: i64) -> Option<LyricDocument> {
        let path = self.cache_path(song_id);
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn exists(&self, song_id: i64) -> bool {
        self.cache_path(song_id).exists()
    }

    pub fn clear(&self) -> Result<(), String> {
        if !self.cache_dir.exists() {
            return Ok(());
        }
        let entries = std::fs::read_dir(&self.cache_dir)
            .map_err(|e| format!("Failed to read cache dir: {}", e))?;
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
        Ok(())
    }
}
