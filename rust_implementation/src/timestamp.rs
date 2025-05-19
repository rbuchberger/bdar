use derive_more::Display;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Unix timestamp
#[derive(Debug, Copy, Clone, Display)]
#[display("{}", self.human_duration_ago())]
pub struct Timestamp(pub u64);

const DAY_IN_SECONDS: usize = 60 * 60 * 24;

// Unix timestamp, in seconds
impl Timestamp {
    pub fn duration_ago(&self) -> Duration {
        SystemTime::now()
            .duration_since(UNIX_EPOCH + Duration::from_secs(self.0))
            .expect("bdar does not support files older than 1970.")
    }

    pub fn human_duration_ago(&self) -> String {
        let mut formatter = timeago::Formatter::new();
        formatter.num_items(2);
        formatter.convert(self.duration_ago())
    }
}

impl From<SystemTime> for Timestamp {
    fn from(time: SystemTime) -> Self {
        Timestamp(
            time.duration_since(UNIX_EPOCH)
                .expect("bdar does not support files older than 1970")
                .as_secs(),
        )
    }
}
