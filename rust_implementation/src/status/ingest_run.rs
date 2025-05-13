use derive_more::Display;

use crate::db::DB;
use crate::timestamp::Timestamp;
use crate::{sql, Result};

use super::StatusErr;

#[derive(Debug, Copy, Clone, Display)]
#[display("{}", self.started)]
pub struct IngestRun {
    pub id: usize,
    pub started: Timestamp,
    pub finished: Option<Timestamp>,
}

impl IngestRun {
    pub fn get_last(db: &DB) -> Result<Self> {
        db.connection
            .query_row(sql!("get_last_ingest_run"), (), |row| {
                Ok(IngestRun {
                    id: row.get::<_, usize>(0)?,
                    started: row.get::<_, u64>(1).map(Timestamp)?,
                    finished: row.get::<_, Option<u64>>(2)?.map(Timestamp),
                })
            })
            .or(Err(Box::new(StatusErr::NotIndexed)))
    }
}
