use std::fs;
use std::path::PathBuf;

use rusqlite::Connection;

macro_rules! sql {
    ($name:literal) => {
        include_str!(concat!("sql/", $name, ".sql"))
    };
}

#[derive(Debug)]
pub struct DB {
    path: PathBuf,
    connection: Connection,
}

#[derive(Debug)]
struct Snapshot {
    id: isize,
    capture_time: isize,
}

#[derive(Debug)]
struct IngestRun {
    id: isize,
    started: isize,
    finished: Option<isize>,
}

#[derive(Debug)]
struct Disk {
    id: isize,
    disk_number: isize,
    snapshot_id: isize,
    capacity: isize,
}

#[derive(Debug)]
struct File {
    id: isize,
    ingest_run_id: isize,
    size: isize,
    name: String,
    modified: isize,
    hash: Option<Vec<u8>>,
    added_snapshot_id: Option<isize>,
    deleted_snapshot_id: Option<isize>,
    disk_id: Option<isize>,
}

impl DB {
    pub fn new(path: &PathBuf) -> Self {
        let connection = Connection::open(&path).unwrap();
        return Self {
            path: path.clone(),
            connection,
        };
    }

    pub fn reset(db: Self) -> Result<(), std::io::Error> {
        fs::remove_file(db.path)
    }

    pub fn initialize(&self) -> Result<(), rusqlite::Error> {
        let result = self.connection.execute_batch(sql!("init"));

        return result;
    }

    pub fn start_ingest_run(&self) -> Result<isize, rusqlite::Error> {
        self.connection
            .query_row(sql!("create_ingest_run"), (), |row| row.get(0))
    }

    pub fn finish_ingest_run(&self, id: isize) -> Result<usize, rusqlite::Error> {
        self.connection
            .execute(sql!("finish_ingest_run"), &[(":ingest_run_id", &id)])
    }
}
