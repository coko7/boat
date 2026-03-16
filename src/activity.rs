use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub type ActId = u64;
pub type ActTime = DateTime<Local>;

pub struct ActivityFactory {
    serial_id: u64,
}

impl ActivityFactory {
    pub fn new() -> Self {
        Self { serial_id: 0 }
    }

    pub fn create_activity(&mut self, name: &str) -> Activity {
        // TODO: comupte a fast hash from "act_name,id"
        // ID should still be a u64. Every new activity will have a new incrmented ID serial.
        // Cross-days, the ID will still be a serial. The first activity of the day may be 323 for
        // example.
        // An activity will also have a hash associated to it, hash should be computed fast.
        // the hash is base64 and should be used in commands to select/filter that activity easy
        // and fast. Similar to git, you only need a prefix of the hash for it to work.
        // As long as the prefix is unique that is.
        // the hash should be bae64 encoded to make it easy to write in command line
        let niu = Activity {
            id: self.get_next_serial_id(),
            parent_id: None,
            name: name.to_string(),
            tags: HashSet::new(),
            tracking: HashSet::new(),
        };

        self.serial_id += 1;
        niu
    }

    fn get_next_serial_id(&mut self) -> ActId {
        let id = self.serial_id;
        self.serial_id += 1;
        id
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Activity {
    id: ActId,
    parent_id: Option<ActId>,
    name: String,
    // description: Option<String>,
    tags: HashSet<String>,
    tracking: HashSet<(ActTime, Option<ActTime>)>,
}

impl Activity {
    pub fn id(&self) -> ActId {
        self.id
    }

    pub fn parent_id(&self) -> Option<ActId> {
        self.parent_id
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

    pub fn hash() -> String {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityDefinition {
    pub id: ActId,
    pub parent_id: Option<ActId>,
    pub name: String,
    #[serde(deserialize_with = "crate::parser::deserialize_hashset")]
    pub tags: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityLog {
    pub id: ActId,
    pub start: ActTime,
    pub end: Option<ActTime>,
}
