use boat_lib::models::log::Log as DatabaseLog;
use serde::{Deserialize, Serialize};
use yansi::Paint;

use crate::{
    models::{FieldSpec, RowPrintable, activity::SimpleActivity, log::PrintableLog},
    utils::{self, date::DateTimeRenderMode},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrintableActivityLog {
    pub log: PrintableLog,
    pub activity: SimpleActivity,
}

impl PrintableActivityLog {
    pub fn from_activity_and_log(activity: &SimpleActivity, log: &DatabaseLog) -> Self {
        Self {
            log: PrintableLog::from_log(log),
            activity: activity.clone(),
        }
    }
}

impl RowPrintable for PrintableActivityLog {
    fn field_specs() -> Vec<FieldSpec> {
        vec![
            ("id", "ID", '>'),
            ("name", "Name", '<'),
            ("description", "Description", '<'),
            ("tags", "Tags", '<'),
            ("start", "Start", '^'),
            ("end", "End", '^'),
            ("duration", "Duration", '<'),
        ]
    }

    fn row_values(&self) -> Vec<String> {
        let dt_render = DateTimeRenderMode::TimeOnly;
        let duration = self.log.duration_sec();

        vec![
            self.activity.id.to_string(),
            self.activity.name.clone(),
            self.activity.description.clone().unwrap_or_default(),
            utils::common::tags_str(&self.activity.tags),
            dt_render.render_date_time(self.log.starts_at),
            self.log
                .ends_at
                .map(|t| dt_render.render_date_time(t))
                .unwrap_or("-".to_string()),
            utils::date::pretty_format_duration(duration, false),
        ]
    }

    fn style_cell(&self, value: String) -> String {
        if self.log.ends_at.is_none() {
            Paint::green(&value).to_string()
        } else {
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boat_lib::models::log::Log as DatabaseLog;
    use chrono::{TimeZone, Utc};
    use std::collections::HashSet;

    fn make_activity() -> SimpleActivity {
        SimpleActivity {
            id: 7,
            name: "testing".to_string(),
            description: Some("a desc".to_string()),
            tags: HashSet::new(),
        }
    }

    fn make_log(ends_after_secs: Option<i64>) -> DatabaseLog {
        let start = Utc.with_ymd_and_hms(2024, 4, 15, 10, 0, 0).unwrap();
        DatabaseLog {
            id: 42,
            activity_id: 7,
            starts_at: start,
            ends_at: ends_after_secs.map(|s| start + chrono::Duration::seconds(s)),
        }
    }

    #[test]
    fn row_values_open_log_shows_dash_for_end() {
        let pal = PrintableActivityLog::from_activity_and_log(&make_activity(), &make_log(None));
        assert_eq!(pal.row_values()[5], "-");
    }

    #[test]
    fn row_values_closed_log_has_non_empty_end() {
        let pal =
            PrintableActivityLog::from_activity_and_log(&make_activity(), &make_log(Some(3600)));
        let end_col = &pal.row_values()[5];
        assert_ne!(end_col, "-");
        assert!(!end_col.is_empty());
    }

    #[test]
    fn row_values_id_and_name_match_activity() {
        let pal =
            PrintableActivityLog::from_activity_and_log(&make_activity(), &make_log(Some(3600)));
        let values = pal.row_values();
        assert_eq!(values[0], "7");
        assert_eq!(values[1], "testing");
    }

    #[test]
    fn style_cell_closed_log_returns_value_unchanged() {
        let pal =
            PrintableActivityLog::from_activity_and_log(&make_activity(), &make_log(Some(3600)));
        let val = "unchanged".to_string();
        assert_eq!(pal.style_cell(val.clone()), val);
    }

    #[test]
    fn style_cell_open_log_result_contains_original_value() {
        let pal = PrintableActivityLog::from_activity_and_log(&make_activity(), &make_log(None));
        let val = "styled".to_string();
        assert!(pal.style_cell(val.clone()).contains(&val));
    }
}
