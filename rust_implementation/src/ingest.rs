// use std::fs::DirEntry;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::context::Context;
use crate::db::{sql, DB};
use crate::Result;

use glob_match::glob_match;
use rusqlite::named_params;
use walkdir::{DirEntry, WalkDir};

// check for interrupted ingest run?
// start ingest run
// walk files
// - handle exclude patterns
// - handle errors (probably permissions)
// - insert into db
// - copy over checksums

// insert performance:
// - use a transaction
// - use prepared statements
// - batching inserts into groups of 5 gives at most a 2x speedup. Not worth it.

pub struct FileInsert {
    pub path: String,
    pub modified: usize,
    pub size: usize,
}

impl FileInsert {
    fn from_entry(entry: walkdir::DirEntry) -> Option<Self> {
        let metadata = match entry.metadata() {
            Err(e) => {
                eprintln!("{}", e);
                None
            }
            Ok(m) => Some(m),
        }?;

        if !metadata.is_file() {
            return None;
        }

        let path = path_to_str(entry.path())?.into();
        let size = metadata.len() as usize;
        let modified = match get_unix_timestamp(metadata.modified()) {
            Ok(time) => Some(time as usize),
            Err(e) => {
                eprintln!(
                    "Error getting timestamp for {}: {}",
                    entry.path().display(),
                    e
                );
                None
            }
        }?;

        return Some(Self {
            path,
            modified,
            size,
        });
    }
}

pub fn index(ctx: &Context, db: &mut DB) -> Result<()> {
    let should_include = |entry: &DirEntry| -> bool {
        match &ctx.repo.exclude_globs {
            None => true,
            Some(globs) => path_to_str(&entry.path()).map_or(false, |path| {
                !globs.iter().any(|glob| glob_match(path, glob))
            }),
        }
    };

    let tx = db.transaction()?;

    let _ = &tx.execute(sql!("create_ingest_run"), ())?;

    let ingest_run_id = tx.last_insert_rowid();

    {
        let mut insert_statement = tx.prepare(sql!("insert_file"))?;

        for entry in WalkDir::new(&ctx.repo.source_dir)
            .into_iter()
            .filter_entry(should_include)
        {
            match entry {
                Ok(entry) => {
                    if let Some(file) = FileInsert::from_entry(entry) {
                        let _ = insert_statement.execute(named_params! {
                            ":path": file.path,
                            ":modified": file.modified,
                            ":size": file.size,
                            ":ingest_run_id": ingest_run_id
                        });
                    }
                }

                Err(err) => {
                    let path = err.path().unwrap_or(Path::new("")).display();

                    println!("failed to access entry {}", path);
                    if let Some(inner) = err.io_error() {
                        match inner.kind() {
                            io::ErrorKind::InvalidData => {
                                eprintln!("file contains invalid data: {}", inner)
                            }
                            io::ErrorKind::PermissionDenied => {
                                eprintln!("Missing permission to read file: {}", inner)
                            }
                            _ => {
                                eprintln!("Unexpected error occurred: {}", inner)
                            }
                        }
                    }
                }
            }
        }
    }

    let _ = &tx.execute(
        sql!("finish_ingest_run"),
        named_params! { ":ingest_run_id": ingest_run_id },
    )?;

    let _ = &tx.commit()?;

    Ok(())
}

fn path_to_str(path: &Path) -> Option<&str> {
    let result = path.to_str();

    if result.is_none() {
        eprintln!("Path is not a valid utf8 string! {}", path.display());
    }

    return result;
}

fn get_unix_timestamp(time: io::Result<SystemTime>) -> Result<u64> {
    Ok(time?.duration_since(UNIX_EPOCH)?.as_secs())
}
