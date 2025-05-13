use std::fs::exists;

use derive_more::{Display, Error};
use rusqlite::{named_params, OptionalExtension};

use crate::context::Context;
use crate::db::DB;
use crate::utils::Timestamp;
use crate::{sql, Result};

#[derive(Debug, Default, Copy, Clone)]
pub struct FileReport {
    pub count: usize,
    pub bytes: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct IngestRun {
    pub id: usize,
    pub started: Timestamp,
    pub finished: Option<Timestamp>,
}

#[derive(Debug, Copy, Clone)]
pub struct Snapshot {
    pub id: usize,
    pub capture_time: Timestamp,
}

#[derive(Debug, Copy, Clone)]
pub struct DirtyItems {
    last_snapshot: Option<Snapshot>,
    last_ingest_run: IngestRun,
    files_missing_checksums: FileReport,
    new_files: FileReport,
    deleted_files: FileReport,
    moved_files: FileReport,
    disks_awaiting_burn: usize,
}

#[derive(Error, Debug, Display)]
pub enum StatusErr {
    NotIndexed,
    NotInitialized,
}

impl DirtyItems {
    fn new(ctx: &Context, db: &DB) -> Result<Self> {
        if !exists(&ctx.db_path)? {
            return Err(Box::new(StatusErr::NotInitialized));
        };

        let last_ingest_run = db
            .connection
            .query_row(sql!("get_last_ingest_run"), (), |row| {
                let finished = row.get::<_, Option<u64>>(2)?;
                Ok(IngestRun {
                    id: row.get::<_, usize>(0)?,
                    started: row.get::<_, u64>(1).map(Timestamp)?,
                    finished: finished.map(Timestamp),
                })
            })
            .or(Err(Box::new(StatusErr::NotIndexed)))?;

        let last_snapshot = db
            .connection
            .query_row(sql!("get_previous_snapshot"), (), |row| {
                Ok(Snapshot {
                    id: row.get::<_, usize>(0)?,
                    capture_time: Timestamp(row.get(1)?),
                })
            })
            .optional()?;

        let files_missing_checksums = db.connection.query_row(
            sql!("report_missing_checksums"),
            named_params! { ":ingest_run_id": last_ingest_run.id },
            |row| {
                Ok(FileReport {
                    count: row.get::<_, usize>(0)?,
                    bytes: row.get::<_, usize>(1)?,
                })
            },
        )?;

        let new_files = db
            .connection
            .query_row(sql!("report_new_files"), (), |row| {
                Ok(FileReport {
                    count: row.get::<_, usize>(0)?,
                    bytes: row.get::<_, usize>(1)?,
                })
            })?;
        dbg!(&new_files);

        let deleted_files = last_snapshot
            .map(|s| {
                db.connection.query_row(
                    sql!("report_deleted_files"),
                    named_params! { ":snapshot_id": s.id},
                    |row| {
                        Ok(FileReport {
                            count: row.get::<_, usize>(0)?,
                            bytes: row.get::<_, usize>(1)?,
                        })
                    },
                )
            })
            .transpose()?
            .unwrap_or(FileReport::default());

        let moved_files = last_snapshot
            .map(|s| {
                db.connection.query_row(
                    sql!("report_moved_files"),
                    named_params! { ":snapshot_id": s.id},
                    |row| {
                        Ok(FileReport {
                            count: row.get::<_, usize>(0)?,
                            bytes: row.get::<_, usize>(1)?,
                        })
                    },
                )
            })
            .transpose()?
            .unwrap_or(FileReport::default());

        let disks_awaiting_burn =
            db.connection
                .query_row(sql!("report_disks_not_burned"), (), |row| {
                    row.get::<_, usize>(0)
                })?;

        dbg!(&disks_awaiting_burn);

        Ok(Self {
            last_snapshot,
            last_ingest_run,
            files_missing_checksums,
            new_files,
            deleted_files,
            moved_files,
            disks_awaiting_burn,
        })
    }
}

pub fn report(ctx: &Context, db: &DB) -> Result<()> {
    match DirtyItems::new(ctx, db) {
        Ok(dirty) => {
            dbg!(dirty);
        }
        Err(problem) => {
            dbg!(problem);
        }
    }
    return Ok(());
}
