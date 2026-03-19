use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub type ActId = String;
pub type ActTime = DateTime<Local>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Activity {
    id: ActId,
    parent_id: Option<ActId>,
    name: String,
    // description: Option<String>,
    tags: HashSet<String>,
    tracking: HashSet<(ActTime, Option<ActTime>)>,
}

impl Activity {
    pub fn from_definition(definition: ActivityDefinition) -> Activity {
        Activity {
            id: definition.id,
            parent_id: definition.parent_id,
            name: definition.name,
            tags: definition.tags,
            tracking: HashSet::new(),
        }
    }

    pub fn new(id: &str, name: &str) -> Activity {
        Activity {
            id: id.to_string(),
            parent_id: None,
            name: name.to_string(),
            tags: HashSet::new(),
            tracking: HashSet::new(),
        }
    }

    pub fn id(&self) -> &ActId {
        &self.id
    }

    pub fn parent_id(&self) -> Option<ActId> {
        self.parent_id.clone()
    }

    pub fn set_parent(&mut self, parent_id: ActId) {
        self.parent_id = Some(parent_id)
    }

    pub fn unset_parent(&mut self) {
        self.parent_id = None;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn tags(&self) -> &HashSet<String> {
        &self.tags
    }

    pub fn set_tags(&mut self, tags: HashSet<String>) {
        self.tags = tags;
    }

    pub fn register_log(&mut self, log: ActivityLog) {
        let log = (log.start, log.end);
        self.tracking.insert(log);
    }

    pub fn tracking(&self) -> &HashSet<(ActTime, Option<ActTime>)> {
        &self.tracking
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ActivityDefinition {
    pub id: ActId,
    pub parent_id: Option<ActId>,
    pub name: String,
    #[serde(deserialize_with = "crate::csv_io::deserialize_hashset")]
    pub tags: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ActivityLog {
    pub id: ActId,
    pub start: ActTime,
    pub end: Option<ActTime>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};
    use std::collections::HashSet;

    #[test]
    fn test_new_activity_and_getters_setters() {
        let mut act = Activity::new("abc", "run");
        assert_eq!(act.id(), "abc");
        assert_eq!(act.parent_id(), None);
        assert_eq!(act.name(), "run");

        act.set_name("walk");
        assert_eq!(act.name(), "walk");

        act.set_parent("parent1".to_string());
        assert_eq!(act.parent_id(), Some("parent1".to_string()));

        act.unset_parent();
        assert_eq!(act.parent_id(), None);

        let mut tags = HashSet::new();
        tags.insert("urgent".to_string());
        act.set_tags(tags.clone());
        assert_eq!(act.tags(), &tags);
    }

    #[test]
    fn test_from_definition() {
        let mut tags = HashSet::new();
        tags.insert("foo".to_string());

        let def = ActivityDefinition {
            id: "id1".to_string(),
            parent_id: Some("pid".to_string()),
            name: "Activity1".to_string(),
            tags: tags.clone(),
        };

        let act = Activity::from_definition(def.clone());

        assert_eq!(act.id(), &def.id);
        assert_eq!(act.parent_id(), def.parent_id);
        assert_eq!(act.name(), &def.name);
        assert_eq!(act.tags(), &tags);
        assert!(act.tracking().is_empty());
    }

    #[test]
    fn test_register_log_and_tracking() {
        let mut act = Activity::new("logid", "move");
        let start = Local.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let end = Some(Local.with_ymd_and_hms(2024, 1, 1, 11, 0, 0).unwrap());
        let log = ActivityLog {
            id: "logid".to_string(),
            start,
            end,
        };

        act.register_log(log.clone());
        assert!(act.tracking().contains(&(log.start, log.end)));
    }
}
