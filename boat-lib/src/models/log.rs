use chrono::{DateTime, Utc};

use crate::repository::Id;

#[derive(Debug, Clone)]
pub struct Log {
    pub id: Id,
    pub activity_id: Id,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct NewLog {
    pub activity_id: Id,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct LogWithActivity {
    pub id: i64,
    pub activity_id: i64,
    pub activity_name: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
}
