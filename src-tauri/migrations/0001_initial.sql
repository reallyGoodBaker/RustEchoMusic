CREATE TABLE tracks (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    artist TEXT,
    album TEXT,
    duration INTEGER NOT NULL,
    path TEXT NOT NULL UNIQUE,
    cover TEXT,
    file_size INTEGER,
    play_count INTEGER DEFAULT 0,
    last_played_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE playlists (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE playlist_tracks (
    id INTEGER PRIMARY KEY,
    playlist_id INTEGER NOT NULL,
    track_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE,
    UNIQUE (playlist_id, track_id),
    UNIQUE (playlist_id, position)
);

CREATE TABLE recent_played (
    track_id INTEGER PRIMARY KEY,
    played_at TEXT NOT NULL,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
);

CREATE TABLE settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    theme TEXT NOT NULL,
    volume INTEGER NOT NULL,
    scan_on_startup INTEGER NOT NULL,
    reduce_motion INTEGER NOT NULL,
    library_dirs TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_tracks_title ON tracks(title);
CREATE INDEX idx_tracks_artist ON tracks(artist);
CREATE INDEX idx_tracks_album ON tracks(album);
CREATE INDEX idx_playlist_tracks_playlist_id ON playlist_tracks(playlist_id);
CREATE INDEX idx_playlist_tracks_track_id ON playlist_tracks(track_id);
CREATE INDEX idx_recent_played_played_at ON recent_played(played_at);
