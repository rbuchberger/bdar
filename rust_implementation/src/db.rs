use std::fs;
use std::path::PathBuf;

use rusqlite::{Connection, Transaction};

use crate::Result;

macro_rules! sql {
    ($name:literal) => {
        include_str!(concat!("sql/", $name, ".sql"))
    };
}

pub(crate) use sql;

#[derive(Debug)]
pub struct DB {
    path: PathBuf,
    connection: Connection,
}

// #[derive(Debug)]
// struct Snapshot {
//     id: isize,
//     capture_time: isize,
// }
//
// #[derive(Debug)]
// struct IngestRun {
//     id: isize,
//     started: isize,
//     finished: Option<isize>,
// }
//
// #[derive(Debug)]
// struct Disk {
//     id: isize,
//     disk_number: isize,
//     snapshot_id: isize,
//     capacity: isize,
// }
//
// #[derive(Debug)]
// struct File {
//     id: isize,
//     ingest_run_id: isize,
//     size: isize,
//     name: String,
//     modified: isize,
//     hash: Option<Vec<u8>>,
//     added_snapshot_id: Option<isize>,
//     deleted_snapshot_id: Option<isize>,
//     disk_id: Option<isize>,
// }

impl DB {
    pub fn new(path: &PathBuf) -> Result<Self> {
        let connection = Connection::open(&path)?;
        return Ok(Self {
            path: path.clone(),
            connection,
        });
    }

    pub fn reset(db: Self) -> Result<()> {
        Ok(fs::remove_file(db.path)?)
    }

    pub fn initialize(&self) -> Result<()> {
        Ok(self.connection.execute_batch(sql!("init"))?)
    }

    // pub fn start_ingest_run(&self) -> Result<usize> {
    //     Ok(self
    //         .connection
    //         .query_row(sql!("create_ingest_run"), (), |row| row.get(0))?)
    // }

    // pub fn finish_ingest_run(&self, id: isize) -> Result<usize> {
    //     Ok(self
    //         .connection
    //         .execute(sql!("finish_ingest_run"), &[(":ingest_run_id", &id)])?)
    // }

    pub fn transaction(&mut self) -> Result<Transaction> {
        Ok(self.connection.transaction()?)
    }

    // pub fn insert_file(&self, file: &FileInsert, ingest_run_id: usize) -> Result<()> {
    //     let mut statement = self.connection.prepare_cached(sql!("insert_file"))?;
    //
    //     statement.execute(named_params! {
    //         ":path": file.path,
    //         ":modified": file.modified,
    //         ":size": file.size,
    //         ":ingest_run_id": ingest_run_id
    //     })?;
    //
    //     Ok(())
    // }
}
