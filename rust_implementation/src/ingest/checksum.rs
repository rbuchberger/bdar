use std::path::PathBuf;

use rusqlite::named_params;

use crate::context::Context;
use crate::db::DB;
use crate::file_entry::FileEntry;
use crate::status::{FileReport, IngestRun};
use crate::{sql, Result};

pub struct Checksum(pub [u8; 32]);

// Early returns an error if there's a problem reading from or writing to the db.
// Errors relating to individual files are logged to STDERR and ignored.
// Metadata is updated if the underlying file has changed since indexing.
// Missing files are deleted from the db (only for that index run).
pub fn generate_checksums(ctx: &Context, db: &mut DB) -> Result<()> {
    let ingest_run_id = IngestRun::get_last(db)?.id;
    let mut statement = db
        .connection
        .prepare(sql!("list_files_missing_checksums"))?;

    while FileReport::missing_checksums(db, ingest_run_id)?.count > 0 {
        let mut rows = statement.query(named_params! {
            ":ingest_run_id": ingest_run_id,
            ":limit": 100
        })?;

        while let Some(row) = rows.next()? {
            let entry = FileEntry::from_row(row)?;
            let entry = entry.validate_against_fs(db)?;

            let Some(entry) = entry else {
                continue;
            };

            if ctx.args.verbose {
                println!("Checksumming file - {}", entry.path.to_string_lossy());
            }

            let checksum = checksum_file(&entry.path);

            let checksum = match checksum {
                Ok(v) => v,
                Err(e) => {
                    eprintln!(
                        "Error generating checksum for file {}",
                        entry.path.to_string_lossy()
                    );
                    eprintln!("{}", e);
                    continue;
                }
            };

            entry.set_db_checksum(checksum, db)?;
        }
    }

    Ok(())
}

fn checksum_file(path: &PathBuf) -> Result<Checksum> {
    let mut hasher = blake3::Hasher::new();
    hasher.update_mmap_rayon(path)?;

    let checksum = hasher.finalize();

    Ok(Checksum(*checksum.as_bytes()))
}
