use anyhow::Result;
use std::ops::Bound::{Excluded, Included};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
};

use crate::{
    activity::{ActId, Activity, ActivityDefinition, ActivityLog},
    parser,
};

#[derive(Debug)]
pub struct Store {
    activities: HashMap<ActId, Activity>,
    index: BTreeMap<ActId, ActId>, // full_key -> full_key (or full_key -> ())
}

impl Store {
    fn new() -> Self {
        Self {
            activities: HashMap::new(),
            index: BTreeMap::new(),
        }
    }

    fn insert(&mut self, key: &str, value: Activity) {
        self.activities.insert(key.to_string(), value);
        self.index.insert(key.to_string(), key.to_string());
    }

    fn load_all_in_memory() -> Result<()> {
        let defs = Store::load_activity_definitions()?;
        let logs = Store::load_activity_logs()?;
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

    fn lookup_activity(&self, id_prefix: &str) -> Option<&Activity> {
        let start = Included(id_prefix.to_owned());
        // build an artificial upper bound: prefix with last char incremented
        let mut upper = id_prefix.to_owned();
        if let Some(last) = upper.pop() {
            upper.push(((last as u8) + 1) as char);
        } else {
            // empty prefix = ambiguous by definition
            return None;
        }
        let end = Excluded(upper);

        let mut iter = self.index.range((start, end));
        let first = iter.next()?;
        if iter.next().is_some() {
            // more than one => not unique
            return None;
        }
        let full_key = first.0;
        self.activities.get(full_key)
    }

    // fn create_activity(&mut self, name: &str, description: &str, tags: &HashSet<String>) {}
    // fn update_activity(&mut self, id: ActivityId) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_activity_with_exact_match_works() {
        let mut store = Store::new();

        let id = "0123456789abcdef";
        let act = Activity::new(&id, "foo");
        store.insert(id, act.clone());

        let res = store.lookup_activity("0123456789abcdef");
        assert_eq!(res, Some(act).as_ref());
    }

    #[test]
    fn lookup_activity_with_unique_short_prefix_works() {
        let mut store = Store::new();

        let id1 = "0123456789abcdef";
        let id2 = "fedcba9876543210";
        let act1 = Activity::new(id1, "foo");
        let act2 = Activity::new(id2, "bar");
        store.insert(id1, act1.clone());
        store.insert(id2, act2.clone());

        let res = store.lookup_activity("01");
        assert_eq!(res, Some(act1).as_ref());

        let res2 = store.lookup_activity("fe");
        assert_eq!(res2, Some(act2).as_ref());
    }

    #[test]
    fn lookup_activity_with_ambiguous_prefix_returns_none() {
        let mut store = Store::new();

        let id1 = "0123456789abcdef";
        let id2 = "0123aaaaaaaaaaaa";
        let act1 = Activity::new(id1, "foo");
        let act2 = Activity::new(id2, "bar");
        store.insert(id1, act1.clone());
        store.insert(id2, act2.clone());

        // prefix "0123" matches both keys
        let res = store.lookup_activity("0123");
        assert!(res.is_none());
    }

    #[test]
    fn lookup_activity_with_no_match_returns_none() {
        let mut store = Store::new();
        let id = "0123456789abcdef";
        store.insert(id, Activity::new(id, "foo"));

        let res = store.lookup_activity("ff");
        assert!(res.is_none());
    }

    #[test]
    fn full_key_still_needs_to_be_unique() {
        let mut store = Store::new();

        let id = "0123456789abcdef";
        let act1 = Activity::new(id, "foo");
        let act2 = Activity::new(id, "bar");
        store.insert(id, act1.clone());
        store.insert(id, act2.clone()); // overwrite previous

        // prefix is still unique and returns the latest value
        let res = store.lookup_activity("0123456789abcdef");
        assert_eq!(res, Some(act2).as_ref());
    }
}
