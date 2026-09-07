use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::audio::state::PlayMode;
use crate::models::Track;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackQueue {
    pub tracks: Vec<Track>,
    pub current_index: Option<usize>,
    pub play_mode: PlayMode,
    pub history: Vec<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueRemoveResult {
    pub play_index: Option<usize>,
    pub should_stop: bool,
}

impl Default for PlaybackQueue {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: None,
            play_mode: PlayMode::ListLoop,
            history: Vec::new(),
        }
    }
}

impl PlaybackQueue {
    pub fn sync(
        &mut self,
        tracks: Vec<Track>,
        current_index: Option<usize>,
        play_mode: PlayMode,
        history: Vec<i64>,
    ) {
        self.tracks = tracks;
        self.current_index = current_index.filter(|i| *i < self.tracks.len());
        self.play_mode = play_mode;
        self.history = history;
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_index = None;
        self.history.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn set_current_index(&mut self, index: Option<usize>) {
        self.current_index = index.filter(|i| *i < self.tracks.len());
    }

    pub fn set_play_mode(&mut self, mode: PlayMode) {
        self.play_mode = mode;
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current_index.and_then(|i| self.tracks.get(i))
    }

    pub fn current_track_cloned(&self) -> Option<Track> {
        self.current_track().cloned()
    }

    pub fn track(&self, index: usize) -> Option<&Track> {
        self.tracks.get(index)
    }

    pub fn require_track(&self, index: usize) -> Result<Track, String> {
        self.tracks
            .get(index)
            .cloned()
            .ok_or_else(|| "Invalid queue index".to_string())
    }

    pub fn contains_track(&self, track_id: i64) -> bool {
        self.tracks.iter().any(|t| t.id == track_id)
    }

    pub fn find_track(&self, track_id: i64) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == track_id)
    }

    pub fn remove_track(&mut self, track_id: i64) -> Option<QueueRemoveResult> {
        let remove_index = self.tracks.iter().position(|track| track.id == track_id)?;

        let was_current = self.current_index == Some(remove_index);

        self.tracks.remove(remove_index);
        self.history.retain(|id| *id != track_id);

        if self.tracks.is_empty() {
            self.current_index = None;
            return Some(QueueRemoveResult {
                play_index: None,
                should_stop: true,
            });
        }

        if was_current {
            let next_index = remove_index.min(self.tracks.len() - 1);
            self.current_index = Some(next_index);
            return Some(QueueRemoveResult {
                play_index: Some(next_index),
                should_stop: false,
            });
        }

        if let Some(current_index) = self.current_index {
            if remove_index < current_index {
                self.current_index = Some(current_index - 1);
            }
        }

        Some(QueueRemoveResult {
            play_index: None,
            should_stop: false,
        })
    }

    pub fn insert_next(&mut self, mut track: Track) {
        track.cover = None;

        if let Some(existing_index) = self.tracks.iter().position(|item| item.id == track.id) {
            if self.current_index == Some(existing_index) {
                return;
            }

            self.tracks.remove(existing_index);

            if let Some(current_index) = self.current_index {
                if existing_index < current_index {
                    self.current_index = Some(current_index - 1);
                }
            }
        }

        if self.tracks.is_empty() || self.current_index.is_none() {
            self.tracks.push(track);
            self.current_index = Some(0);
            return;
        }

        let insert_index = self.current_index.unwrap() + 1;

        self.tracks.insert(insert_index, track);
    }

    pub fn insert_tracks_as_next(&mut self, tracks: Vec<Track>) {
        for track in tracks.into_iter().rev() {
            self.insert_next(track);
        }
    }

    pub fn replace_playlist(
        &mut self,
        tracks: Vec<Track>,
        target_id: i64,
    ) -> Result<usize, String> {
        self.tracks = tracks;
        self.history.clear();
        let index = self
            .tracks
            .iter()
            .position(|t| t.id == target_id)
            .unwrap_or(0);
        self.current_index = Some(index);
        Ok(index)
    }

    pub fn next_index(&self) -> Option<usize> {
        if self.tracks.is_empty() {
            return None;
        }

        let current = self.current_index.unwrap_or(0);

        match self.play_mode {
            PlayMode::SingleLoop => Some(current),

            PlayMode::ListLoop => Some((current + 1) % self.tracks.len()),

            PlayMode::Shuffle => {
                if self.tracks.len() <= 1 {
                    return Some(0);
                }

                let mut rng = rand::rng();

                loop {
                    let next = rng.random_range(0..self.tracks.len());

                    if next != current {
                        return Some(next);
                    }
                }
            }
        }
    }

    pub fn previous_index(&self) -> Option<usize> {
        if self.tracks.is_empty() {
            return None;
        }

        if let Some(last_id) = self.history.last() {
            if let Some(index) = self.tracks.iter().position(|track| track.id == *last_id) {
                return Some(index);
            }
        }

        let current = self.current_index.unwrap_or(0);

        Some(if current == 0 {
            self.tracks.len() - 1
        } else {
            current - 1
        })
    }

    pub fn move_next(&mut self) -> Option<usize> {
        if let Some(track) = self.current_track() {
            if self.history.last() != Some(&track.id) {
                self.history.push(track.id);
            }
        }

        self.next_index()
    }

    pub fn move_previous(&mut self) -> Option<usize> {
        let index = self.previous_index()?;

        if let Some(track) = self.tracks.get(index) {
            if self.history.last() == Some(&track.id) {
                self.history.pop();
            }
        }

        Some(index)
    }

    pub fn payload(&self) -> Self {
        self.clone()
    }
}
