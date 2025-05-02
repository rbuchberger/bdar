CREATE TABLE IF NOT EXISTS files (
  file_id INTEGER PRIMARY KEY,
  added_snapshot_id INTEGER,
  deleted_snapshot_id INTEGER,
  disk_number INTEGER,
  size INTEGER NOT NULL,
  modified INTEGER NOT NULL,
  parent TEXT NOT NULL,
  stem TEXT NOT NULL,
  extension TEXT,
  hash BLOB,

  FOREIGN KEY (added_snapshot_id) REFERENCES snapshots (snapshot_id),
  FOREIGN KEY (deleted_snapshot_id) REFERENCES snapshots (snapshot_id)
);
