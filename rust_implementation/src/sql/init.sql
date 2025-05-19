CREATE TABLE snapshots (
    id INTEGER NOT NULL PRIMARY KEY,
    capture_time INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE ingest_runs (
    id INTEGER NOT NULL PRIMARY KEY,
    started INTEGER NOT NULL DEFAULT (unixepoch()),
    finished INTEGER
);

CREATE TABLE disks (
    id INTEGER NOT NULL PRIMARY KEY,
    disk_number INTEGER NOT NULL,
    snapshot_id INTEGER NOT NULL,
    capacity INTEGER NOT NULL,
    burned_timestamp INTEGER,
    FOREIGN KEY (snapshot_id) REFERENCES snapshots (id)
);

CREATE TABLE files (
    id INTEGER PRIMARY KEY,
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
    checksum BLOB,
    FOREIGN KEY (added_snapshot_id) REFERENCES snapshots (id),
    FOREIGN KEY (deleted_snapshot_id) REFERENCES snapshots (id),
    FOREIGN KEY (disk_id) REFERENCES disks (id),
    FOREIGN KEY (ingest_run_id) REFERENCES ingest_runs (id)
);
