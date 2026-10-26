use crate::context::Context;
use crate::db::sql;

use glob_match::glob_match;
use rusqlite::{named_params, Transaction};
use walkdir::WalkDir;

use super::index::{InsertableFile, SkipReason};

// Either handles its own errors, or panics.
pub fn walk_tree(ctx: &Context, tx: &Transaction, ingest_run_id: usize) {
    let mut insert_statement = tx
        .prepare(sql!("insert_file"))
        .expect("Problem preparing hardcoded SQL statement");

    WalkDir::new(&ctx.repo.source_dir)
        .into_iter()
        .filter_entry(|entry| match &ctx.repo.exclude_globs {
            None => true,
            Some(globs) => entry
                .path()
                .to_str()
                .is_some_and(|path| !globs.iter().any(|glob| glob_match(path, glob))),
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
