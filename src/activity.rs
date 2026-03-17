use chrono::{DateTime, Local};
use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::utils;

pub type ActId = String;
pub type ActTime = DateTime<Local>;

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
    pub fn from_definition(definition: ActivityDefinition) -> Activity {
        Activity {
            id: definition.id,
            parent_id: definition.parent_id,
            name: definition.name,
            tags: definition.tags,
            tracking: HashSet::new(),
        }
    }

    pub fn new(name: String) -> Activity {
        let id = Self::generate_rand_hash();
        Activity {
            id,
            parent_id: None,
            name,
            tags: HashSet::new(),
            tracking: HashSet::new(),
        }
    }

    fn generate_rand_hash() -> ActId {
        let random_input: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        utils::compute_hex_hash(&random_input)
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
