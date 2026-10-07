use boat_lib::{models::activity::Activity, repository::Id};
use chrono::{DateTime, Local, TimeDelta, Utc};

pub struct ReportEntry {
    pub activity_id: Id,
    pub name: String,
    pub duration: TimeDelta,
    pub ongoing: bool,
}

pub struct DayReport {
    /// Entries sorted by time spent, largest first
    pub entries: Vec<ReportEntry>,
    pub total: TimeDelta,
}

impl DayReport {
    /// Time spent on each activity since local midnight.
    /// Logs crossing midnight only count the part that falls on today.
    pub fn today(activities: &[Activity], now: DateTime<Utc>) -> Self {
        let day_start = now
            .with_timezone(&Local)
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .and_then(|midnight| midnight.and_local_timezone(Local).earliest())
            .map(|midnight| midnight.with_timezone(&Utc))
            .unwrap_or(now);

        let mut entries: Vec<ReportEntry> = activities
            .iter()
            .filter_map(|activity| {
                let mut ongoing = false;
                let duration = activity
                    .logs
                    .iter()
                    .map(|log| {
                        ongoing |= log.ends_at.is_none();
                        let start = log.starts_at.max(day_start);
                        let end = log.ends_at.unwrap_or(now).min(now);
                        (end - start).max(TimeDelta::zero())
                    })
                    .sum::<TimeDelta>();

                (duration > TimeDelta::zero()).then(|| ReportEntry {
                    activity_id: activity.id,
                    name: activity.name.clone(),
                    duration,
                    ongoing,
                })
            })
            .collect();

        entries.sort_by_key(|e| std::cmp::Reverse(e.duration));
        let total = entries.iter().map(|e| e.duration).sum();
        Self { entries, total }
    }

    pub fn share(&self, entry: &ReportEntry) -> f64 {
        if self.total.is_zero() {
            return 0.0;
        }
        entry.duration.num_milliseconds() as f64 / self.total.num_milliseconds() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boat_lib::models::log::Log;
    use std::collections::HashSet;

    fn activity(id: Id, logs: Vec<(DateTime<Utc>, Option<DateTime<Utc>>)>) -> Activity {
        Activity {
            id,
            name: format!("activity {id}"),
            description: None,
            tags: HashSet::new(),
            logs: logs
                .into_iter()
                .enumerate()
                .map(|(i, (starts_at, ends_at))| Log {
                    id: i as Id,
                    activity_id: id,
                    starts_at,
                    ends_at,
                })
                .collect(),
        }
    }

    fn local_today(h: u32, m: u32) -> DateTime<Utc> {
        Local::now()
            .date_naive()
            .and_hms_opt(h, m, 0)
            .unwrap()
            .and_local_timezone(Local)
            .earliest()
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn sums_today_and_clips_logs_crossing_midnight() {
        let now = local_today(12, 0);
        let activities = vec![
            activity(1, vec![(local_today(9, 0), Some(local_today(10, 0)))]),
            // started yesterday 23:00, ended today 01:00 -> only 1h counts
            activity(
                2,
                vec![(
                    local_today(0, 0) - TimeDelta::hours(1),
                    Some(local_today(1, 0)),
                )],
            ),
            // ongoing since 11:00 -> 1h
            activity(3, vec![(local_today(11, 0), None)]),
            // yesterday only -> excluded
            activity(
                4,
                vec![(
                    local_today(0, 0) - TimeDelta::hours(5),
                    Some(local_today(0, 0) - TimeDelta::hours(4)),
                )],
            ),
            // never started -> excluded
            activity(5, vec![]),
        ];

        let report = DayReport::today(&activities, now);
        assert_eq!(report.entries.len(), 3);
        assert_eq!(report.total, TimeDelta::hours(3));
        assert!(
            report
                .entries
                .iter()
                .all(|e| e.duration == TimeDelta::hours(1))
        );
        assert!(
            report
                .entries
                .iter()
                .find(|e| e.activity_id == 3)
                .unwrap()
                .ongoing
        );
        let share = report.share(&report.entries[0]);
        assert!((share - 1.0 / 3.0).abs() < 1e-9);
    }
}
