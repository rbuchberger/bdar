use derive_more::Display;
use rusqlite::OptionalExtension;

use crate::db::DB;
use crate::timestamp::Timestamp;
use crate::{sql, Result};

#[derive(Debug, Copy, Clone, Display)]
#[display("Number {id} - captured {capture_time} ago")]
pub struct Snapshot {
    pub id: usize,
    pub capture_time: Timestamp,
}

impl Snapshot {
    pub fn get_last(db: &DB) -> Result<Option<Self>> {
        Ok(db
            .connection
            .query_row(sql!("get_previous_snapshot"), (), |row| {
                Ok(Snapshot {
                    id: row.get::<_, usize>(0)?,
                    capture_time: Timestamp(row.get(1)?),
                })
            })
            .optional()?)
    }
}
