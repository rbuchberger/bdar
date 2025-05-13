use crate::context::Context;
use crate::db::{sql, DB};
use crate::timestamp::Timestamp;
use crate::{Error, Result};

use derive_more::{Display, From};
use glob_match::glob_match;
use rusqlite::{named_params, Transaction};
use walkdir::WalkDir;

// check for interrupted ingest run?
// start ingest run
// walk files
// - copy over checksums

// insert performance:
// - use a transaction
// - use prepared statements
// - batching inserts into groups of 5 gives at most a 2x speedup. Not worth it.

pub struct InsertableFile {
    pub path: String,
    pub modified: usize,
    pub size: usize,
}

#[derive(Debug, Display, From)]
pub enum SkipReason {
    NotAFile,
    InvalidPath(String),
    OtherError(Error),
}

impl TryFrom<walkdir::DirEntry> for InsertableFile {
    type Error = SkipReason;

    fn try_from(entry: walkdir::DirEntry) -> core::result::Result<Self, SkipReason> {
        let metadata = entry
            .metadata()
            .map_err(|e| SkipReason::OtherError(e.into()))?;

        if !metadata.is_file() {
            return Err(SkipReason::NotAFile)?;
        }

        let path = entry
            .path()
            .to_str()
            .ok_or_else(|| SkipReason::InvalidPath(entry.path().display().to_string()))?
            .to_string();

        let size = metadata.len() as usize;
        let modified = metadata
            .modified()
            .map_err(|e| SkipReason::OtherError(e.into()))?;

        let modified = Timestamp::try_from(modified)
            .map_err(|e| SkipReason::OtherError(e.into()))?
            .0 as usize;

        return Ok(Self {
            path,
            modified,
            size,
        });
    }
}

pub fn index(ctx: &Context, db: &mut DB) -> Result<()> {
    let tx = db.transaction()?;
    let ingest_run_id: usize = tx.query_row(sql!("create_ingest_run"), [], |r| r.get(0))?;

    println!("Indexing directory: {}", ctx.repo.source_dir);

    walk_tree(ctx, &tx, ingest_run_id);

    // mark ingest run finished
    assert!(&tx
        .execute(
            sql!("finish_ingest_run"),
            named_params! { ":ingest_run_id": ingest_run_id },
        )?
        .eq(&1),);

    // copy checksums
    let _ = &tx.execute(
        sql!("copy_checksums"),
        named_params! { ":ingest_run_id": ingest_run_id },
    )?;

    // clean up old files
    let _ = &tx.execute(
        sql!("cleanup_files"),
        named_params! { ":ingest_run_id": ingest_run_id },
    );

    tx.commit().map_err(|e| e.into())
}

// Either handles its own errors, or panics.
fn walk_tree(ctx: &Context, tx: &Transaction, ingest_run_id: usize) {
    let mut insert_statement = tx
        .prepare(sql!("insert_file"))
        .expect("Problem preparing hardcoded SQL statement");

    WalkDir::new(&ctx.repo.source_dir)
        .into_iter()
        .filter_entry(|entry| match &ctx.repo.exclude_globs {
            None => true,
            Some(globs) => entry.path().to_str().map_or(false, |path| {
                !globs.iter().any(|glob| glob_match(path, glob))
            }),
        })
        .filter_map(|entry| {
            entry
                .map_err(|e| eprintln!("File indexing error: {}", e))
                .ok()
        })
        .filter_map(|entry| {
            InsertableFile::try_from(entry)
                .map_err(|error| match error {
                    SkipReason::NotAFile => (),
                    SkipReason::InvalidPath(p) => {
                        eprintln!("File indexing error: Not a UTF-8 filename: {}", p);
                    }
                    SkipReason::OtherError(e) => {
                        eprintln!("File indexing error: {}", e)
                    }
                })
                .ok()
        })
        .for_each({
            |file| {
                if ctx.args.verbose {
                    println!("Indexing file: {}", file.path)
                }

                insert_statement
                    .execute(named_params! {
                        ":path": file.path,
                        ":modified": file.modified,
                        ":size": file.size,
                        ":ingest_run_id": ingest_run_id
                    })
                    .expect("Error inserting file into database.");
            }
        });
}
