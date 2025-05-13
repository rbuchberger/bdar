use derive_more::Display;
use humansize::{format_size, BINARY};
use rusqlite::{named_params, Params};

use crate::db::DB;
use crate::{sql, Result};

#[derive(Debug, Default, Copy, Clone, Display)]
#[display("{} ({})", self.count, self.human_bytes())]
pub struct FileReport {
    pub count: usize,
    pub bytes: usize,
}

impl FileReport {
    pub fn human_bytes(&self) -> String {
        format_size(self.bytes, BINARY)
    }

    pub fn missing_checksums(db: &DB, ingest_run_id: usize) -> Result<Self> {
        Self::query(
            db,
            sql!("report_missing_checksums"),
            named_params! { ":ingest_run_id": ingest_run_id },
        )
    }

    pub fn new_files(db: &DB) -> Result<Self> {
        Self::query(db, sql!("report_new_files"), ())
    }

    pub fn deleted_files(db: &DB, snapshot_id: usize) -> Result<Self> {
        Self::query(
            db,
            sql!("report_deleted_files"),
            named_params! { ":snapshot_id": snapshot_id },
        )
    }

    pub fn moved_files(db: &DB, snapshot_id: usize) -> Result<Self> {
        Self::query(
            db,
            sql!("report_moved_files"),
            named_params! { ":snapshot_id": snapshot_id },
        )
    }

    fn query<T: Params>(db: &DB, sql: &str, params: T) -> Result<Self> {
        Ok(db.connection.query_row(sql, params, |row| {
            Ok(Self {
                count: row.get(0)?,
                bytes: row.get(1)?,
            })
        })?)
    }
}
