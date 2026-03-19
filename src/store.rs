use anyhow::Result;
use std::collections::{BTreeMap, HashMap};
use std::ops::Bound::{Excluded, Included};

use crate::activity::{ActId, Activity, ActivityDefinition, ActivityLog};
use crate::converter;
use crate::repository::Repository;

#[derive(Debug)]
pub struct ActivityStore {
    defs_repo: Box<dyn Repository<ActivityDefinition>>,
    logs_repo: Box<dyn Repository<ActivityLog>>,
    activities: HashMap<ActId, Activity>,
    id_lookup: BTreeMap<ActId, ActId>, // full_key -> full_key (or full_key -> ())
}

impl ActivityStore {
    pub fn new(
        defs_repo: Box<dyn Repository<ActivityDefinition>>,
        logs_repo: Box<dyn Repository<ActivityLog>>,
    ) -> Self {
        Self {
            defs_repo,
            logs_repo,
            activities: HashMap::new(),
            id_lookup: BTreeMap::new(),
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.defs_repo.initialize()?;
        self.logs_repo.initialize()?;

        let defs = self.defs_repo.load_all()?;
        let logs = self.logs_repo.load_all()?;
        let activities = converter::recreate_activities(defs, logs)?;

        self.activities = activities;
        self.rebuild_lookup_table()?;
        Ok(())
    }

    fn rebuild_lookup_table(&mut self) -> Result<()> {
        self.id_lookup = BTreeMap::new();
        for (key, _) in self.activities.iter() {
            self.id_lookup.insert(key.to_string(), key.to_string());
        }
        Ok(())
    }

    pub fn insert(&mut self, key: &str, value: Activity) {
        self.activities.insert(key.to_string(), value);
        self.id_lookup.insert(key.to_string(), key.to_string());
    }

    pub fn lookup_activity(&self, id_prefix: &str) -> Option<&Activity> {
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

        let mut iter = self.id_lookup.range((start, end));
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

    #[derive(Debug)]
    struct TestRepo {}

    impl<T: std::fmt::Debug> Repository<T> for TestRepo {
        fn initialize(&self) -> Result<()> {
            todo!()
        }

        fn load_all(&self) -> Result<Vec<T>> {
            todo!()
        }

        fn save_all(&self, _data: &[T]) -> Result<()> {
            todo!()
        }
    }

    fn create_test_store() -> ActivityStore {
        ActivityStore {
            activities: HashMap::new(),
            id_lookup: BTreeMap::new(),
            defs_repo: Box::new(TestRepo {}),
            logs_repo: Box::new(TestRepo {}),
        }
    }

    #[test]
    fn lookup_activity_with_exact_match_works() {
        let mut store = create_test_store();

        let id = "0123456789abcdef";
        let act = Activity::new(&id, "foo");
        store.insert(id, act.clone());

        let res = store.lookup_activity("0123456789abcdef");
        assert_eq!(res, Some(act).as_ref());
    }

    #[test]
    fn lookup_activity_with_unique_short_prefix_works() {
        let mut store = create_test_store();

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
        let mut store = create_test_store();

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
        let mut store = create_test_store();
        let id = "0123456789abcdef";
        store.insert(id, Activity::new(id, "foo"));

        let res = store.lookup_activity("ff");
        assert!(res.is_none());
    }

    #[test]
    fn full_key_still_needs_to_be_unique() {
        let mut store = create_test_store();

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
