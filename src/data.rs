use std::collections::HashMap;

use crate::activity::{ActId, Activity};

#[derive(Debug)]
pub struct ActivityStore {
    activities: HashMap<ActId, Activity>,
}

impl ActivityStore {
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
