use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use crate::context::Context;
use crate::db::DB;

use walkdir::WalkDir;

pub fn index(ctx: &Context, db: &DB) {
    // let ingest_run_id = db.start_ingest_run().unwrap();
    // interesting fields:
    // entry.path() - full path
    // entry.metadata().unwrap().modified() - System timestamp 

    for entry in WalkDir::new(ctx.repo.source_dir.clone())
        .into_iter()
        .take(10)
    {
        let entry = entry.unwrap();
        println!(
            "{:?} - {:?}",
            &entry.path(),
            &entry.metadata().unwrap().len()
        )
    }
}
