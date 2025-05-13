use derive_more::Display;

use crate::Result;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Unix timestamp
#[derive(Debug, Copy, Clone, Display)]
#[display("{}", self.human_duration_ago())]
pub struct Timestamp(pub u64);

const DAY_IN_SECONDS: usize = 60 * 60 * 24;

// Unix timestamp, in seconds
impl Timestamp {
    pub fn from(time: SystemTime) -> Result<Self> {
        Ok(Timestamp(time.duration_since(UNIX_EPOCH)?.as_secs()))
    }
    pub fn duration_ago(&self) -> Result<Duration> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH + Duration::from_secs(self.0))
            .map_err(|e| e.into())
    }

    pub fn human_duration_ago(&self) -> String {
        let mut formatter = timeago::Formatter::new();
        formatter.num_items(2);
        formatter.convert(self.duration_ago().unwrap_or_default())
    }
}

impl TryFrom<SystemTime> for Timestamp {
    fn try_from(time: SystemTime) -> Result<Self> {
        Ok(Timestamp(time.duration_since(UNIX_EPOCH)?.as_secs()))
    }

    type Error = crate::Error;
}
