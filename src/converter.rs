use anyhow::{Context, Result, bail};
use log::warn;
use std::collections::HashMap;

use crate::activity::{ActId, Activity, ActivityDefinition, ActivityLog};

pub fn recreate_activities(
    definitions: Vec<ActivityDefinition>,
    logs: Vec<ActivityLog>,
) -> Result<HashMap<ActId, Activity>> {
    let mut activities = init_activities_from_definitions(definitions)?;
    for log in logs {
        let activity = activities
            .get_mut(&log.id)
            .with_context(|| format!("activity not found: {}", log.id))?;

        // FIX: Verify no overlap in logs
        activity.register_log(log);
    }

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
