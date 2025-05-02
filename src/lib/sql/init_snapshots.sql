CREATE TABLE snapshots(
  snapshot_id INTEGER NOT NULL PRIMARY KEY,
  capture_time INTEGER NOT NULL DEFAULT (unixepoch())
)
