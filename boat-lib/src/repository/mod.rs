use anyhow::Result;
use chrono::{DateTime, Utc};
use std::fmt::Debug;

pub mod activities_repository;
pub mod logs_repository;
pub mod tags_repository;

pub type Id = i64;

pub trait Repository<T: RepositoryItem>: Debug {
    fn create(&mut self, item: T) -> Result<Id>;
    fn get(&self, id: Id) -> Result<Option<T>>;
    fn list(&self) -> Result<Vec<T>>;
    fn update(&mut self, item: T) -> Result<()>;
    fn delete(&mut self, id: Id) -> Result<()>;
}

pub trait RepositoryItem {
    fn id(&self) -> Id;
    fn set_id(&mut self, value: Id);
}

pub fn parse_datetime(value: Option<String>) -> rusqlite::Result<Option<DateTime<Utc>>> {
    match value {
        Some(s) => {
            // SQLite format: "2026-03-22 21:47:00" → add T/Z for RFC3339
            let rfc3339 = format!("{}Z", s.replace(" ", "T"));
            let dt = DateTime::parse_from_rfc3339(&rfc3339)
                .map_err(|e| {
                    eprintln!("Parse error on '{}': {}", s, e);
                    rusqlite::Error::InvalidQuery
                })?
                .with_timezone(&Utc);
            Ok(Some(dt))
        }
        None => Ok(None),
    }
}
