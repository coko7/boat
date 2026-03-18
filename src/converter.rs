use anyhow::{Context, Result, bail};
use log::warn;
use std::collections::HashMap;

use crate::activity::{ActId, Activity, ActivityDefinition, ActivityLog};

pub fn recreate_activities(
    definitions: Vec<ActivityDefinition>,
    logs: Vec<ActivityLog>,
) -> Result<HashMap<ActId, Activity>> {
    let activities = init_activities_from_definitions(definitions)?;
    let activities = apply_tracking_to_activities(activities, logs)?;
    Ok(activities)
}

fn init_activities_from_definitions(
    definitions: Vec<ActivityDefinition>,
) -> Result<HashMap<ActId, Activity>> {
    let mut activities = HashMap::new();

    for definition in definitions {
        if let Some(existing) = activities.get(&definition.id) {
            bail!(
                "duplicate ID found: trying to register {definition:?} while {existing:?} already exists"
            );
        }

        if let Some(ref parent_id) = definition.parent_id
            && *parent_id == definition.id
        {
            bail!("activity cannot be its own parent: {definition:?}");
        }

        // FIX: Need to check for cycle refs:
        // TaskA --parent-> TaskB
        // TaskB --parent-> TaskC
        // TaskC --parent-> TaskA
        warn!("experimental: cycle refs have not been checked");

        let id = definition.id.clone();
        let niu = Activity::from_definition(definition);
        activities.insert(id, niu);
    }

    Ok(activities)
}

fn apply_tracking_to_activities(
    mut activities: HashMap<ActId, Activity>,
    logs: Vec<ActivityLog>,
) -> Result<HashMap<ActId, Activity>> {
    let mut previous_log: Option<ActivityLog> = None;
    for log in logs {
        let activity = activities
            .get_mut(&log.id)
            .with_context(|| format!("activity not found: {}", log.id))?;

        if let Some(end) = log.end {
            if log.start > end {
                bail!("activity log has an impossible start time: {log:?}");
            }
        }

        if let Some(previous_log) = &previous_log {
            let prev_end = previous_log.end.with_context(|| {
                format!("found a previous log with no end time: {previous_log:?}")
            })?;

            if log.start < prev_end {
                bail!(
                    "activity log {log:?} cannot start before the previous one ended: {previous_log:?}"
                );
            }
        }

        activity.register_log(log.clone());
        previous_log = Some(log);
    }

    Ok(activities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::init_test_logger;
    use chrono::{Local, TimeZone};
    use std::collections::HashSet;

    fn create_test_act_definition(id: &str, parent_id: Option<&str>) -> ActivityDefinition {
        ActivityDefinition {
            id: id.to_string(),
            parent_id: parent_id.map(|s| s.to_string()),
            name: format!("Activity {id}"),
            tags: HashSet::new(),
        }
    }

    fn create_test_act_log(id: &str, start_secs: i64, end_secs: Option<i64>) -> ActivityLog {
        ActivityLog {
            id: id.to_string(),
            start: Local.timestamp_opt(start_secs, 0).unwrap(),
            end: end_secs.map(|e| Local.timestamp_opt(e, 0).unwrap()),
        }
    }

    #[test]
    fn recreate_activities_basic_no_logs() {
        init_test_logger();
        let defs = vec![
            create_test_act_definition("foo", None),
            create_test_act_definition("bar", None),
        ];
        let map = recreate_activities(defs, vec![]).expect("should succeed");

        assert_eq!(map.len(), 2);
        assert!(map.contains_key("foo"));
        assert!(map.contains_key("bar"));
    }

    #[test]
    fn recreate_activities_log_with_start_greater_than_end_should_error() {
        init_test_logger();
        let defs = vec![create_test_act_definition("foo", None)];
        let logs = vec![create_test_act_log("foo", 200, Some(100))]; // start > end
        let err = recreate_activities(defs, logs).unwrap_err();
        assert!(err.to_string().contains("impossible start time"));
    }

    #[test]
    fn recreate_activities_log_starts_before_previous_ends_should_error() {
        init_test_logger();
        let defs = vec![create_test_act_definition("foo", None)];
        let logs = vec![
            create_test_act_log("foo", 100, Some(200)),
            create_test_act_log("foo", 150, Some(250)), // 150 < 200
        ];
        let err = recreate_activities(defs, logs).unwrap_err();
        assert!(
            err.to_string()
                .contains("cannot start before the previous one ended")
        );
    }

    #[test]
    fn recreate_activities_previous_log_missing_end_should_error() {
        init_test_logger();
        let defs = vec![create_test_act_definition("foo", None)];
        let logs = vec![
            create_test_act_log("foo", 100, None), // missing end
            create_test_act_log("foo", 200, Some(300)),
        ];
        let err = recreate_activities(defs, logs).unwrap_err();
        assert!(
            err.to_string()
                .contains("found a previous log with no end time")
        );
    }

    #[test]
    fn recreate_activities_definition_with_tags_should_preserve_tags() {
        init_test_logger();
        use std::collections::HashSet;
        let mut tags = HashSet::new();
        tags.insert("code".to_string());
        tags.insert("cli".to_string());
        let def = ActivityDefinition {
            id: "foo".to_string(),
            parent_id: None,
            name: "has tags".to_string(),
            tags: tags.clone(),
        };
        let defs = vec![def];
        let map = recreate_activities(defs, vec![]).expect("should succeed");
        let foo = map.get("foo").unwrap();
        assert_eq!(foo.tags(), &tags);
    }

    #[test]
    fn recreate_activities_multiple_logs_for_same_activity_should_accumulate_tracking() {
        init_test_logger();
        let defs = vec![create_test_act_definition("foo", None)];
        let logs = vec![
            create_test_act_log("foo", 100, Some(200)),
            create_test_act_log("foo", 250, Some(300)),
        ];
        let map = recreate_activities(defs, logs.clone()).expect("should succeed");
        let foo = map.get("foo").unwrap();
        let expected: Vec<(
            chrono::DateTime<chrono::Local>,
            Option<chrono::DateTime<chrono::Local>>,
        )> = logs.iter().map(|log| (log.start, log.end)).collect();
        for pair in expected {
            assert!(foo.tracking().contains(&pair));
        }
        assert_eq!(foo.tracking().len(), 2);
    }

    #[test]
    fn recreate_activities_parent_id_not_present_in_defs_should_create_ok() {
        init_test_logger();
        // parent_id references unknown activity, should create
        let def = create_test_act_definition("foo", Some("not_in_defs"));
        let defs = vec![def];
        let map = recreate_activities(defs, vec![]).expect("should succeed");
        let foo = map.get("foo").unwrap();
        assert_eq!(foo.parent_id(), Some("not_in_defs".to_string()));
    }

    #[test]
    fn recreate_activites_log_missing_end_should_be_accepted() {
        init_test_logger();
        let defs = vec![create_test_act_definition("foo", None)];
        let logs = vec![create_test_act_log("foo", 100, None)]; // open event
        let map = recreate_activities(defs.clone(), logs.clone()).expect("should succeed");
        let foo = map.get("foo").unwrap();
        assert!(foo.tracking().contains(&(logs[0].start, logs[0].end)));
    }

    #[test]
    fn recreate_activities_large_batch() {
        init_test_logger();
        let mut defs = Vec::new();
        let mut logs = Vec::new();
        // Only one activity at a time, global sequential logs
        for i in 0..50 {
            let id = format!("id{}", i);
            defs.push(create_test_act_definition(&id, None));
            let start = 100 + i as i64 * 100;
            let end = start + 40;
            logs.push(create_test_act_log(&id, start, Some(end)));
        }
        // Ensure logs are strictly ordered by start time for global exclusive policy
        logs.sort_by(|a, b| a.start.cmp(&b.start));
        let map = recreate_activities(defs.clone(), logs.clone()).expect("should succeed");
        assert_eq!(map.len(), 50);
        for log in &logs {
            let act = map.get(&log.id).unwrap();
            assert!(act.tracking().contains(&(log.start, log.end)));
        }
    }

    #[test]
    fn recreate_activties_logs_with_non_sequential_start_times_should_error() {
        init_test_logger();
        let defs = vec![create_test_act_definition("foo", None)];
        let logs = vec![
            create_test_act_log("foo", 100, Some(200)),
            create_test_act_log("foo", 50, Some(150)), // second starts before first
        ];
        let err = recreate_activities(defs, logs).unwrap_err();
        assert!(
            err.to_string()
                .contains("cannot start before the previous one ended")
        );
    }

    #[test]
    fn recreate_activities_basic_with_logs() {
        init_test_logger();
        let defs = vec![create_test_act_definition("foo", None)];
        let logs = vec![create_test_act_log("foo", 100, Some(200))];
        let map = recreate_activities(defs.clone(), logs.clone()).expect("should succeed");

        let foo = map.get("foo").unwrap();
        assert!(foo.tracking().contains(&(logs[0].start, logs[0].end)));
    }

    #[test]
    fn recreate_activities_log_for_missing_id() {
        init_test_logger();
        let defs = vec![create_test_act_definition("foo", None)];
        let logs = vec![create_test_act_log("bar", 0, None)];

        let err = recreate_activities(defs, logs).unwrap_err();
        assert!(err.to_string().contains("activity not found: bar"));
    }

    #[test]
    fn init_duplicate_id_should_err() {
        init_test_logger();
        let defs = vec![
            create_test_act_definition("foo", None),
            create_test_act_definition("foo", None),
        ];

        let err = super::init_activities_from_definitions(defs).unwrap_err();
        assert!(err.to_string().contains("duplicate ID found"));
    }

    #[test]
    fn init_self_parent_should_err() {
        init_test_logger();
        let defs = vec![create_test_act_definition("foo", Some("foo"))];

        let err = super::init_activities_from_definitions(defs).unwrap_err();
        assert!(err.to_string().contains("cannot be its own parent"));
    }

    #[test]
    fn cycle_detection_placeholder_warning() {
        // Cannot test for actual cycle handling until it's implemented,
        // but warn! macro should not panic.
        // Just ensure we can construct nested parent-child without crash.
        init_test_logger();
        let defs = vec![
            create_test_act_definition("a", Some("b")),
            create_test_act_definition("b", Some("c")),
            create_test_act_definition("c", Some("a")),
        ];
        // Should NOT fail due to cycle (yet) but still create 3 activities.
        let map = super::init_activities_from_definitions(defs).expect("no cycle detection yet");
        assert_eq!(map.len(), 3);
    }
}
