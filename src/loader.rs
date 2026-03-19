use anyhow::Result;
use std::fmt::Debug;

use crate::activity::{ActivityDefinition, ActivityLog};

pub trait DataLoader: Debug {
    fn initialize(&self) -> Result<()>;
    fn load_all_activity_definitions(&self) -> Result<Vec<ActivityDefinition>>;
    fn load_all_activity_logs(&self) -> Result<Vec<ActivityLog>>;
}
