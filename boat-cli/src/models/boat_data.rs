use boat_lib::{
    models::{activity::Activity as DatabaseActivity, log::Log as DatabaseLog},
    repository::Id,
};
use log::info;
use std::{cmp::Reverse, collections::HashMap};

use crate::{
    cli::{self},
    models::{
        activity::{PrintableActivity, SimpleActivity},
        activity_log::PrintableActivityLog,
    },
    utils::{self, date::DateTimeRenderMode},
};

pub struct BoatData {
    pub activities: HashMap<Id, SimpleActivity>,
    pub logs: HashMap<Id, Vec<DatabaseLog>>,
}

impl BoatData {
    pub fn create_filtered_data(
        db_activities: Vec<DatabaseActivity>,
        period_input: cli::PeriodInput,
    ) -> Self {
        info!("creating filtered boat data");
        let activities = db_activities
            .iter()
            .map(|db_act| (db_act.id, SimpleActivity::from_db_activity(db_act)))
            .collect();

        info!("filtering activity logs based on date filters (period / dates)");
        let logs = db_activities
            .into_iter()
            .map(|db_act| {
                let filtered_logs = db_act
                    .logs
                    .into_iter()
                    .filter(|log| utils::common::matches_period_filter(log, &period_input))
                    .collect();
                (db_act.id, filtered_logs)
            })
            .collect();

        Self { activities, logs }
    }

    pub fn get_printable_activities(&self) -> Vec<PrintableActivity> {
        info!("retrieving printable activities");
        let mut prt_acts: Vec<_> = self
            .activities
            .values()
            .map(|act| {
                let related_logs = self.logs.get(&act.id).map(Vec::as_slice).unwrap_or(&[]);
                PrintableActivity::from_activity_and_logs(act, related_logs)
            })
            .collect();

        info!("sorting printable activities by duration DESC");
        prt_acts.sort_by_key(|act| Reverse(act.duration));
        prt_acts
    }

    pub fn get_printable_logs(&self) -> Vec<PrintableActivityLog> {
        info!("retrieving printable logs");
        let mut prt_logs: Vec<_> = self
            .logs
            .values()
            .flatten()
            .map(|log| {
                let act = self.activities.get(&log.activity_id).unwrap();
                PrintableActivityLog::from_activity_and_log(act, log)
            })
            .collect();

        info!("sorting printable logs by start dt ASC");
        prt_logs.sort_by_key(|al| al.log.starts_at);
        prt_logs
    }

    pub fn to_csv_str(
        &self,
        include_instructions: bool,
        include_activity_definitions: bool,
    ) -> String {
        let mut csv_data = String::new();

        if include_instructions {
            csv_data.push_str("# This is a CSV export of your activities and logs.\n");
            csv_data
                .push_str("# Lines starting with '#' are comments and should not be modified.\n");
            csv_data.push_str(
            "# Activity definitions are included for reference but are not meant to be edited.\n",
        );
            csv_data.push_str("# You may only edit activity logs here.\n#\n");
        }

        if include_activity_definitions {
            csv_data.push_str("# Activity definitions:\n");
            csv_data.push_str("# | ID | Name | Description | Tags |\n");
            csv_data.push_str("# | -- | ---- | ----------- | ---- |\n");
        }

        let mut activities = self.activities.values().collect::<Vec<_>>();
        activities.sort_by_key(|act| act.id);

        let mut act_logs = vec![];
        for act in activities {
            let related_logs = self.logs.get(&act.id).map(Vec::as_slice).unwrap_or(&[]);
            if related_logs.is_empty() {
                continue;
            }

            for log in related_logs {
                let local_starts_at = log.starts_at.with_timezone(&chrono::Local);
                let local_ends_at = log.ends_at.map(|dt| dt.with_timezone(&chrono::Local));
                act_logs.push((act.id, log.id, local_starts_at, local_ends_at));
            }

            if !include_activity_definitions {
                continue;
            }

            let act_csv = format!(
                "# | {} | {} | {} | {} |\n",
                act.id,
                act.name,
                act.description.clone().unwrap_or("".to_string()),
                utils::common::tags_str(&act.tags)
            );
            csv_data.push_str(&act_csv);
        }

        if include_activity_definitions {
            csv_data.push_str("\n\n");
        }

        if include_instructions {
            csv_data.push_str(
                "# Below are your activity logs. You can edit the start and end times here.\n",
            );
            csv_data.push_str(
            "# If you want to mark the latest activity as ongoing, simply remove the end time (leave it blank) for that log.\n",
        );
            csv_data.push_str(
                "# Please keep the activity_id and log_id unchanged to avoid breaking the data.\n#\n",
            );
        }

        csv_data.push_str("# Logs (activity_id,log_id,starts_at,ends_at):\n");
        csv_data.push_str("# ===== EDIT DATA BELOW =====\n");

        act_logs.sort_by_key(|(_, _, starts_at, _)| *starts_at);
        for (act_id, log_id, starts_at, ends_at) in act_logs {
            let ends_at = match ends_at {
                Some(dt) => DateTimeRenderMode::DateAndTime.render_date_time(dt),
                None => "".to_string(),
            };

            let log_csv = format!(
                "{},{},{},{}\n",
                act_id,
                log_id,
                DateTimeRenderMode::DateAndTime.render_date_time(starts_at),
                ends_at
            );
            csv_data.push_str(&log_csv);
        }

        csv_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boat_lib::models::log::Log as DatabaseLog;
    use chrono::{TimeZone, Utc};
    use std::collections::HashMap;

    fn make_activity(id: Id, name: &str) -> SimpleActivity {
        SimpleActivity {
            id,
            name: name.to_string(),
            description: None,
            tags: std::collections::HashSet::new(),
        }
    }

    fn make_log(id: Id, activity_id: Id, start_h: u32, end_h: Option<u32>) -> DatabaseLog {
        let base = Utc.with_ymd_and_hms(2024, 4, 15, start_h, 0, 0).unwrap();
        DatabaseLog {
            id,
            activity_id,
            starts_at: base,
            ends_at: end_h.map(|h| Utc.with_ymd_and_hms(2024, 4, 15, h, 0, 0).unwrap()),
        }
    }

    fn boat_data(acts: Vec<(Id, &str)>, logs: Vec<(Id, Vec<DatabaseLog>)>) -> BoatData {
        let activities: HashMap<Id, SimpleActivity> = acts
            .into_iter()
            .map(|(id, name)| (id, make_activity(id, name)))
            .collect();
        let logs: HashMap<Id, Vec<DatabaseLog>> = logs.into_iter().collect();
        BoatData { activities, logs }
    }

    // --- get_printable_activities ---

    #[test]
    fn get_printable_activities_sorted_by_duration_desc() {
        let data = boat_data(
            vec![(1, "short"), (2, "long")],
            vec![
                (1, vec![make_log(10, 1, 10, Some(11))]), // 1h
                (2, vec![make_log(20, 2, 8, Some(12))]),  // 4h
            ],
        );
        let acts = data.get_printable_activities();
        assert_eq!(acts.len(), 2);
        assert!(acts[0].duration > acts[1].duration);
        assert_eq!(acts[0].name, "long");
    }

    #[test]
    fn get_printable_activities_activity_with_no_logs_has_zero_duration() {
        let data = boat_data(vec![(1, "idle")], vec![(1, vec![])]);
        let acts = data.get_printable_activities();
        assert_eq!(acts[0].duration, 0);
        assert!(!acts[0].ongoing);
    }

    // --- get_printable_logs ---

    #[test]
    fn get_printable_logs_sorted_by_start_asc() {
        let data = boat_data(
            vec![(1, "work")],
            vec![(
                1,
                vec![
                    make_log(2, 1, 14, Some(15)), // 14:00
                    make_log(1, 1, 9, Some(10)),  // 09:00 — earlier
                ],
            )],
        );
        let logs = data.get_printable_logs();
        assert_eq!(logs.len(), 2);
        assert!(logs[0].log.starts_at < logs[1].log.starts_at);
    }

    // --- to_csv_str ---

    fn data_line_count(csv: &str) -> usize {
        csv.lines()
            .filter(|l| !l.is_empty() && !l.trim_start().starts_with('#'))
            .count()
    }

    #[test]
    fn to_csv_str_contains_log_line() {
        let data = boat_data(
            vec![(1, "coding")],
            vec![(1, vec![make_log(1, 1, 10, Some(11))])],
        );
        let csv = data.to_csv_str(false, false);
        assert_eq!(data_line_count(&csv), 1);
        let data_line = csv
            .lines()
            .find(|l| !l.trim_start().starts_with('#'))
            .unwrap();
        // format: act_id,log_id,starts_at,ends_at
        let fields: Vec<&str> = data_line.split(',').collect();
        assert_eq!(
            fields.len(),
            4,
            "log line should have 4 comma-separated fields"
        );
        assert_eq!(fields[0], "1");
        assert_eq!(fields[1], "1");
    }

    #[test]
    fn to_csv_str_open_ended_log_has_empty_ends_at() {
        let data = boat_data(
            vec![(1, "coding")],
            vec![(1, vec![make_log(1, 1, 10, None)])],
        );
        let csv = data.to_csv_str(false, false);
        let data_line = csv
            .lines()
            .find(|l| !l.trim_start().starts_with('#'))
            .unwrap();
        // last field before newline should be empty
        assert!(
            data_line.ends_with(','),
            "ends_at should be empty for open-ended log"
        );
    }

    #[test]
    fn to_csv_str_includes_instructions_when_requested() {
        let data = boat_data(
            vec![(1, "t")],
            vec![(1, vec![make_log(1, 1, 10, Some(11))])],
        );
        let with = data.to_csv_str(true, false);
        let without = data.to_csv_str(false, false);
        assert!(with.contains("# This is a CSV export"));
        assert!(!without.contains("# This is a CSV export"));
    }

    #[test]
    fn to_csv_str_includes_activity_definitions_when_requested() {
        let data = boat_data(
            vec![(1, "t")],
            vec![(1, vec![make_log(1, 1, 10, Some(11))])],
        );
        let with = data.to_csv_str(false, true);
        let without = data.to_csv_str(false, false);
        assert!(with.contains("# Activity definitions:"));
        assert!(!without.contains("# Activity definitions:"));
    }

    #[test]
    fn to_csv_str_logs_sorted_chronologically() {
        let data = boat_data(
            vec![(1, "work")],
            vec![(
                1,
                vec![
                    make_log(2, 1, 14, Some(15)), // later
                    make_log(1, 1, 9, Some(10)),  // earlier
                ],
            )],
        );
        let csv = data.to_csv_str(false, false);
        let data_lines: Vec<&str> = csv
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect();
        assert_eq!(data_lines.len(), 2);
        // first line should be log id=1 (9:00), second should be log id=2 (14:00)
        assert!(
            data_lines[0].starts_with("1,1,"),
            "earlier log should appear first"
        );
        assert!(
            data_lines[1].starts_with("1,2,"),
            "later log should appear second"
        );
    }

    #[test]
    fn to_csv_str_activity_with_no_logs_is_omitted() {
        let data = boat_data(vec![(1, "idle")], vec![(1, vec![])]);
        let csv = data.to_csv_str(false, false);
        assert_eq!(data_line_count(&csv), 0);
    }
}
