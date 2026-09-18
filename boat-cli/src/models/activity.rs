use boat_lib::models::{activity::Activity as DatabaseActivity, log::Log as DatabaseLog};
use boat_lib::repository::Id;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::models::{FieldSpec, RowPrintable};
use crate::utils;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimpleActivity {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub tags: HashSet<String>,
}

impl SimpleActivity {
    pub fn from_db_activity(activity: &DatabaseActivity) -> Self {
        Self {
            id: activity.id,
            name: activity.name.clone(),
            description: activity.description.clone(),
            tags: activity.tags.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrintableActivity {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub duration: i64,
    pub ongoing: bool,
    pub tags: HashSet<String>,
}

impl PrintableActivity {
    pub fn from_activity_and_logs(activity: &SimpleActivity, logs: &[DatabaseLog]) -> Self {
        let total_duration_sec: i64 = logs.iter().map(Self::count_log_duration_sec).sum();

        Self {
            id: activity.id,
            name: activity.name.clone(),
            description: activity.description.clone(),
            ongoing: logs.iter().any(|l| l.ends_at.is_none()),
            duration: total_duration_sec,
            tags: activity.tags.clone(),
        }
    }

    fn count_log_duration_sec(log: &DatabaseLog) -> i64 {
        let start = log.starts_at;
        let end = log.ends_at.unwrap_or(Utc::now());
        (end - start).num_seconds()
    }
}

impl RowPrintable for PrintableActivity {
    fn field_specs() -> Vec<FieldSpec> {
        vec![
            ("id", "ID", '>'),
            ("name", "Name", '<'),
            ("description", "Description", '<'),
            ("tags", "Tags", '<'),
            ("duration", "Duration", '<'),
        ]
    }

    fn row_values(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.name.clone(),
            self.description.clone().unwrap_or_default(),
            utils::common::tags_str(&self.tags),
            utils::date::pretty_format_duration(self.duration, false),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boat_lib::models::log::Log as DatabaseLog;
    use chrono::{TimeZone, Utc};

    fn simple_act(id: Id) -> SimpleActivity {
        SimpleActivity {
            id,
            name: "coding".to_string(),
            description: None,
            tags: HashSet::new(),
        }
    }

    fn closed_log(id: Id, duration_secs: i64) -> DatabaseLog {
        let start = Utc.with_ymd_and_hms(2024, 4, 15, 10, 0, 0).unwrap();
        DatabaseLog {
            id,
            activity_id: 1,
            starts_at: start,
            ends_at: Some(start + chrono::Duration::seconds(duration_secs)),
        }
    }

    fn open_log(id: Id) -> DatabaseLog {
        DatabaseLog {
            id,
            activity_id: 1,
            starts_at: Utc.with_ymd_and_hms(2024, 4, 15, 10, 0, 0).unwrap(),
            ends_at: None,
        }
    }

    #[test]
    fn tags_str_renders_comma_separated() {
        let mut tags = HashSet::new();
        tags.insert("foo".to_owned());
        tags.insert("bar".to_owned());

        let act = PrintableActivity {
            id: 42,
            name: "n".to_owned(),
            description: None,
            duration: 0,
            ongoing: false,
            tags,
        };
        let tags_str = utils::common::tags_str(&act.tags);

        assert!(tags_str.contains("foo"));
        assert!(tags_str.contains("bar"));
        assert!(tags_str.find(',').is_some());
    }

    #[test]
    fn from_activity_and_logs_empty_logs_zero_duration_not_ongoing() {
        let act = simple_act(1);
        let pa = PrintableActivity::from_activity_and_logs(&act, &[]);
        assert_eq!(pa.duration, 0);
        assert!(!pa.ongoing);
    }

    #[test]
    fn from_activity_and_logs_single_closed_log() {
        let act = simple_act(1);
        let pa = PrintableActivity::from_activity_and_logs(&act, &[closed_log(1, 3600)]);
        assert_eq!(pa.duration, 3600);
        assert!(!pa.ongoing);
    }

    #[test]
    fn from_activity_and_logs_open_ended_log_is_ongoing() {
        let act = simple_act(1);
        let pa = PrintableActivity::from_activity_and_logs(&act, &[open_log(1)]);
        assert!(pa.ongoing);
    }

    #[test]
    fn from_activity_and_logs_multiple_logs_sums_durations() {
        let act = simple_act(1);
        let logs = [closed_log(1, 1800), closed_log(2, 900)];
        let pa = PrintableActivity::from_activity_and_logs(&act, &logs);
        assert_eq!(pa.duration, 2700);
        assert!(!pa.ongoing);
    }

    #[test]
    fn from_activity_and_logs_mixed_closed_and_open_is_ongoing() {
        let act = simple_act(1);
        let logs = [closed_log(1, 1800), open_log(2)];
        let pa = PrintableActivity::from_activity_and_logs(&act, &logs);
        assert!(pa.ongoing);
    }
}
