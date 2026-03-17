use anyhow::Result;
use std::{collections::HashMap, fs};

use crate::{
    activity::{ActId, Activity, ActivityDefinition, ActivityLog},
    parser,
};

#[derive(Debug)]
pub struct ActivityStore {
    activities: HashMap<ActId, Activity>,
}

impl ActivityStore {
    fn load_all_in_memory() -> Result<()> {
        let defs = ActivityStore::load_activity_definitions()?;
        let logs = ActivityStore::load_activity_logs()?;
        Ok(())
    }

    fn load_activity_definitions() -> Result<Vec<ActivityDefinition>> {
        let raw_defs = fs::read_to_string("data/boat_defs.txt")?;
        let act_defs = parser::parse_csv::<ActivityDefinition>(&raw_defs)?;
        Ok(act_defs)
    }

    fn load_activity_logs() -> Result<Vec<ActivityLog>> {
        let raw_logs = fs::read_to_string("data/boat_logs.txt")?;
        let act_logs = parser::parse_csv::<ActivityLog>(&raw_logs)?;
        Ok(act_logs)
    }
    // fn create_activity(&mut self, name: &str, description: &str, tags: &HashSet<String>) {}
    // fn update_activity(&mut self, id: ActivityId) {}
}

// pub fn load_from_file() -> Result<Vec<Activity>> {
//     let mut rdr = Reader::from_path("data/acts.csv")?;
//     for result in rdr.records() {
//         let record = result?;
//         println!("{:?}", record); // Vec<String> per row
//     }
//
//     for line in reader.lines() {
//         let line = line?;
//         let act_record = parse_activity_record_line(&line)?;
//         println!("{}", line);
//     }
//
//     todo!()
// }
