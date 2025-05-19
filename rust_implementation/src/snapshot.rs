use crate::context::Context;
use crate::db::DB;
use crate::timestamp::Timestamp;
use crate::{sql, Result};

pub struct Snapshot {
    id: u64,
    capture_time: Timestamp,
}

impl Snapshot {
    fn build(ctx: &Context, db: &mut DB) -> Result<()> {
        let tx = db.transaction()?;

        // Create snapshot record
        let snap = tx.query_row(sql!("create_snapshot"), (), |row| {
            Ok(Snapshot {
                id: row.get(0)?,
                capture_time: row.get::<_, u64>(1).map(Timestamp)?,
            })
        })?;
        // Mark deleted files
        // - find files that are not present in current run, but have not been deleted
        // - mark them as deleted in current snapshot
        // Mark no-ops
        // - Find files that haven't changed
        // - delete them from current run
        // Mark updated files
        // Mark moved files
        // Mark new files
        // Assign disk numbers

        Ok(())
    }
}
