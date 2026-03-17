use anyhow::{Result, bail};
use std::collections::HashSet;

use crate::activity::ActivityDefinition;

pub fn verify_activity_definitions(act_defs: &[ActivityDefinition]) -> Result<()> {
    let mut ids = HashSet::new();

    for def in act_defs {
        if !ids.insert(def.id.clone()) {
            bail!("validation failed: found duplicate ID: {}", def.id);
        }
    }

    Ok(())
}
