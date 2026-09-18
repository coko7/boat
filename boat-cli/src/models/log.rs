use boat_lib::models::log::Log as DatabaseLog;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrintableLog {
    pub starts_at: DateTime<Local>,
    pub ends_at: Option<DateTime<Local>>,
}

impl PrintableLog {
    pub fn from_log(log: &DatabaseLog) -> Self {
        Self {
            starts_at: log.starts_at.with_timezone(&Local),
            ends_at: log.ends_at.map(|t| t.with_timezone(&Local)),
        }
    }
    pub fn duration_sec(&self) -> i64 {
        let end = self.ends_at.unwrap_or(Local::now());
        (end - self.starts_at).num_seconds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_sec() {
        let now = Local::now();
        let log = PrintableLog {
            starts_at: now,
            ends_at: Some(now + chrono::Duration::seconds(60)),
        };
        assert_eq!(log.duration_sec(), 60);

        let log = PrintableLog {
            starts_at: now,
            ends_at: None,
        };
        assert!(log.duration_sec() >= 0); // Should not panic
    }
}
