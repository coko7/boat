use std::collections::HashSet;

use crate::models::log::Log;

#[derive(Debug, Clone)]
pub struct Activity {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub tags: HashSet<String>,
    pub logs: Vec<Log>,
}

#[derive(Debug)]
pub struct NewActivity {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

// Helper struct for base activity data
#[derive(Debug)]
pub struct ActivityBase {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}
