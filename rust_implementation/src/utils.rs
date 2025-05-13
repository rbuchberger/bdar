use crate::Result;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_unix_timestamp(time: SystemTime) -> Result<Timestamp> {
    Ok(Timestamp(time.duration_since(UNIX_EPOCH)?.as_secs()))
}

// Unix timestamp
#[derive(Debug, Copy, Clone)]
pub struct Timestamp(pub u64);
