use std::fmt;
use std::fs::exists;

use derive_more::{Display, Error};

use crate::context::Context;
use crate::db::DB;
use crate::Result;

use self::disks::Disks;
use self::files::FileReport;
use self::ingest_run::IngestRun;
use self::snapshot::Snapshot;

mod disks;
mod files;
mod ingest_run;
mod snapshot;

#[derive(Debug, Copy, Clone)]
pub struct Status {
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

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Status Report:")?;
        match self.last_snapshot {
            Some(s) => writeln!(f, "  Last Snapshot: {}", s),
            None => writeln!(f, "  Last Snapshot: Never"),
        }?;
        writeln!(f, "  Last Index: {}", self.last_ingest_run)?;
        writeln!(f, "  Burn Queue Length: {}", self.disks_awaiting_burn)?;
        writeln!(
            f,
            "  Files Missing Checksums: {}",
            self.files_missing_checksums
        )?;

        if self.files_missing_checksums.count > 0 {
            writeln!(
                f,
                "    - Note that without a checksum, bdar cannot detect a moved file."
            )?;
            writeln!(
                f,
                "      It will be reported as deleted and created until hashed."
            )?;
        }

        writeln!(f, "Since Last Snapshot:")?;
        writeln!(f, "  New Files:     {}", self.new_files)?;
        writeln!(f, "  Deleted Files: {}", self.deleted_files)?;
        writeln!(f, "  Moved Files:   {}", self.moved_files)?;

        Ok(())
    }
}

impl Status {
    fn new(ctx: &Context, db: &DB) -> Result<Self> {
        if !exists(&ctx.db_path)? {
            return Err(Box::new(StatusErr::NotInitialized));
        };

        let last_ingest_run = IngestRun::get_last(db)?;
        let last_snapshot = Snapshot::get_last(db)?;
        let files_missing_checksums = FileReport::missing_checksums(db, last_ingest_run.id)?;
        let new_files = FileReport::new_files(db)?;
        let deleted_files = last_snapshot
            .map(|s| FileReport::deleted_files(db, s.id))
            .unwrap_or(Ok(FileReport::default()))?;

        let moved_files = last_snapshot
            .map(|s| FileReport::moved_files(db, s.id))
            .unwrap_or(Ok(FileReport::default()))?;

        let disks_awaiting_burn = Disks::awaiting_burn(db)?;

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
    match Status::new(ctx, db) {
        Ok(status) => {
            println!("{}", status);
        }
        Err(problem) => {
            dbg!(problem);
        }
    }
    return Ok(());
}
