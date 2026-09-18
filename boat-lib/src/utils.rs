use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use rusqlite::Connection;
use std::path::Path;

/// Parses naive datetime string to local timezone.
/// Handles DST gaps/folds by picking earliest valid.
pub fn parse_local_dt(s: &str) -> Result<DateTime<Local>> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
        .with_context(|| format!("failed to parse '{s}' as naive datetime"))?;

    Local
        .from_local_datetime(&naive)
        .earliest()
        .context("invalid local time (e.g., DST gap)")
}

pub fn init_database(db_path: impl AsRef<Path>) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    let schema_sql = include_str!("schema.sql");
    conn.execute_batch(schema_sql)?;
    Ok(conn)
}

#[cfg(test)]
pub fn init_test_logger() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();
}
