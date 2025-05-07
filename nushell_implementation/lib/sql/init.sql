CREATE TABLE snapshots (
    snapshot_id INTEGER NOT NULL PRIMARY KEY,
    capture_time INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE ingest_runs (
    ingest_run_id INTEGER NOT NULL PRIMARY KEY,
    started INTEGER NOT NULL DEFAULT (unixepoch()),
    finished INTEGER
);

CREATE TABLE disks (
    disk_id INTEGER NOT NULL PRIMARY KEY,
    disk_number INTEGER NOT NULL,
    snapshot_id INTEGER NOT NULL,
    capacity INTEGER NOT NULL,
    FOREIGN KEY (snapshot_id) REFERENCES snapshots (snapshot_id)
);

CREATE TABLE IF NOT EXISTS files (
    file_id INTEGER PRIMARY KEY,
    added_snapshot_id INTEGER,
    deleted_snapshot_id INTEGER,
    disk_id INTEGER,
    ingest_run_id INTEGER NOT NULL,
    size INTEGER NOT NULL,
    name TEXT NOT NULL,
    modified INTEGER NOT NULL,
    -- parent TEXT NOT NULL,
    -- stem TEXT NOT NULL,
    -- extension TEXT,
    hash BLOB,
    FOREIGN KEY (added_snapshot_id) REFERENCES snapshots (snapshot_id),
    FOREIGN KEY (deleted_snapshot_id) REFERENCES snapshots (snapshot_id),
    FOREIGN KEY (disk_id) REFERENCES disks (disk_id),
    FOREIGN KEY (ingest_run_id) REFERENCES ingest_runs (ingest_run_id)
);
