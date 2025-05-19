use std::fs::Metadata;
use std::path::PathBuf;

use anyhow::anyhow;
use rusqlite::{named_params, Row};

use crate::db::DB;
use crate::ingest::Checksum;
use crate::timestamp::Timestamp;
use crate::{sql, Result};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub id: u64,
    pub path: PathBuf,
    pub modified: u64,
    pub size: u64,
}

impl PartialEq<Metadata> for FileEntry {
    fn eq(&self, other: &Metadata) -> bool {
        let other_modified = Timestamp::from(
            other
                .modified()
                .expect("bdar does not work on filesystems that don't support mtime"),
        )
        .0;
        self.size == other.len() && self.modified == other_modified
    }
}

impl FileEntry {
    // Parse entry from a sql row
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            path: PathBuf::from(row.get::<_, String>(1)?),
            modified: row.get(2)?,
            size: row.get(3)?,
        })
    }

    // - Only passes up database access errors.
    // - File read errors are printed to stderr, and a none type is returned (so the caller can skip
    //   that particular file.)
    pub fn validate_against_fs(&self, db: &DB) -> Result<Option<FileEntry>> {
        match std::fs::metadata(&self.path) {
            Ok(meta) if *self != meta => {
                println!(
                    "File has been modified since last indexed. Updating database - {}",
                    &self.path.to_string_lossy()
                );

                let entry = self.recreate_from_metadata(meta);
                self.update_db(db)?;

                Ok(Some(entry))
            }
            Ok(_) => Ok(Some(self.clone())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "File has been deleted since last indexed. Updating database - {}",
                    &self.path.to_string_lossy()
                );

                self.delete_from_db(db)?;
                Ok(None)
            }
            Err(e) => {
                eprintln!(
                    "Error trying to access file. Skipping - {}",
                    &self.path.to_string_lossy()
                );

                eprintln!("{}", e);
                Ok(None)
            }
        }
    }

    fn recreate_from_metadata(&self, meta: Metadata) -> Self {
        Self {
            id: self.id,
            path: self.path.clone(),
            size: meta.len(),
            modified: Timestamp::from(
                meta.modified()
                    .expect("bdar requires a filesystem that supports modified time"),
            )
            .0,
        }
    }

    fn update_db(&self, db: &DB) -> Result<()> {
        let affected = db.connection.execute(
            sql!("update_file_meta"),
            named_params! {
                ":id": self.id,
                ":size": self.size,
                ":modified": self.modified
            },
        )?;

        match affected {
            0 => Err(anyhow!("No file entry found for ID number {} - {}", self.id, self.path.to_string_lossy())),
            1 => Ok(()),
            _ => Err(anyhow!("Somehow more than one file was found with the same ID: {}. That should not be possible.", self.id)),
        }
    }

    pub fn set_db_checksum(&self, checksum: Checksum, db: &DB) -> Result<()> {
        let affected = db.connection.execute(
            sql!("set_checksum"),
            named_params! {
                ":id": self.id,
                ":checksum": checksum.0,
            },
        )?;

        match affected {
            0 => Err(anyhow!("No file entry found for ID number {}", self.id)),
            1 => Ok(()),
            _ => Err(anyhow!("Somehow more than one file was found with the same ID: {}. That should not be possible.", self.id)),
        }
    }

    fn delete_from_db(&self, db: &DB) -> Result<()> {
        let affected = db
            .connection
            .execute(sql!("delete_file"), named_params! { ":file_id": self.id })?;

        match affected {
        0 => Err(anyhow!("No file entry found for ID number {}", self.id)),
        1 => Ok(()),
        _ => Err(anyhow!(
            "Somehow more than one file was found with the same ID: {}. That should not be possible. Good luck debugging it because they've been deleted now!",
            self.id
        )),
    }
    }
}
