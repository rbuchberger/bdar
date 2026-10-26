use crate::db::DB;
use crate::{sql, Result};

pub struct Disks;

impl Disks {
    pub fn awaiting_burn(db: &DB) -> Result<usize> {
        Ok(db
            .connection
            .query_row(sql!("report_disks_not_burned"), (), |row| {
                row.get::<_, usize>(0)
            })?)
    }
}
