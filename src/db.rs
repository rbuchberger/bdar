use std::fs;
use std::path::PathBuf;

use rusqlite::{Connection, Transaction};

use crate::Result;

macro_rules! sql {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src",
            "/sql/",
            $name,
            ".sql"
        ))
    };
}

pub(crate) use sql;

#[derive(Debug)]
pub struct DB {
    path: PathBuf,
    pub connection: Connection,
}

impl DB {
    pub fn new(path: &PathBuf) -> Result<Self> {
        Ok(Self {
            path: path.clone(),
            connection: Connection::open(path)?,
        })
    }

    pub fn reset(db: Self) -> Result<()> {
        Ok(fs::remove_file(db.path)?)
    }

    pub fn initialize(&self) -> Result<()> {
        Ok(self.connection.execute_batch(sql!("init"))?)
    }

    pub fn transaction(&mut self) -> Result<Transaction> {
        Ok(self.connection.transaction()?)
    }

    pub fn is_initialized(&self) -> Result<bool> {
        let count: usize = self
            .connection
            .query_row(sql!("count_tables"), (), |row| row.get(0))?;

        Ok(count > 0)
    }
}
