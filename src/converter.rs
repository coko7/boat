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

        if let Some(ref parent_id) = definition.parent_id {
            if *parent_id == definition.id {
                bail!("activity cannot be its own parent: {definition:?}");
            }
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
