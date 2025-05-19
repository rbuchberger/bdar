use crate::context::Context;
use crate::db::{sql, DB};
use crate::ingest::walk_tree::walk_tree;
use crate::timestamp::Timestamp;
use crate::{Error, Result};

use derive_more::{Display, From};
use rusqlite::named_params;

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

        let modified = Timestamp::from(modified).0 as usize;

        Ok(Self {
            path,
            modified,
            size,
        })
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
