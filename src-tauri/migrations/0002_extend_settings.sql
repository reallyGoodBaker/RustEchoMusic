ALTER TABLE settings ADD COLUMN use_album_artist_grouping INTEGER NOT NULL DEFAULT 0;
ALTER TABLE settings ADD COLUMN plugin_dirs TEXT NOT NULL DEFAULT '[]';
ALTER TABLE settings ADD COLUMN plugin_dev_mode INTEGER NOT NULL DEFAULT 0;
ALTER TABLE settings ADD COLUMN plugin_scan_on_startup INTEGER NOT NULL DEFAULT 1;
ALTER TABLE settings ADD COLUMN plugin_log_level TEXT NOT NULL DEFAULT 'warn';
