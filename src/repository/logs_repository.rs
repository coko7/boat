use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result};

use crate::{
    models::log::{Log, LogWithActivity, NewLog},
    repository::{self, Id},
};

pub fn create(conn: &Connection, new_log: NewLog) -> Result<Log> {
    conn.execute(
        "INSERT INTO logs (activity_id, starts_at, ends_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![
            new_log.activity_id,
            new_log.starts_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            new_log
                .ends_at
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
        ],
    )?;

    let id = conn.last_insert_rowid();
    get_by_id(conn, id)
}

pub fn get_by_id(conn: &Connection, id: Id) -> Result<Log> {
    conn.query_row(
        "SELECT id, activity_id, starts_at, ends_at FROM logs WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            let starts_at_str = row
                .get::<_, Option<String>>(2)?
                .expect("Database corruption: missing starts_at");
            let ends_at_str = row.get::<_, Option<String>>(3)?;

            Ok(Log {
                id: row.get(0)?,
                activity_id: row.get(1)?,
                starts_at: repository::parse_datetime(Some(starts_at_str))
                    .expect("Invalid starts_at datetime")
                    .unwrap(),
                ends_at: repository::parse_datetime(ends_at_str)?,
            })
        },
    )
}

pub fn get_for_activity(conn: &Connection, activity_id: Id) -> Result<Vec<Log>> {
    let mut stmt = conn.prepare(
        "SELECT id, activity_id, starts_at, ends_at 
             FROM logs WHERE activity_id = ?1 ORDER BY starts_at DESC",
    )?;

    let logs = stmt
        .query_map(rusqlite::params![activity_id], |row| {
            let starts_at_str = row
                .get::<_, Option<String>>(2)?
                .expect("Database corruption: missing starts_at");
            let ends_at_str = row.get::<_, Option<String>>(3)?;

            Ok(Log {
                id: row.get(0)?,
                activity_id: row.get(1)?,
                starts_at: repository::parse_datetime(Some(starts_at_str))
                    .expect("Invalid starts_at datetime in database")
                    .expect("should not be null"),
                ends_at: repository::parse_datetime(ends_at_str)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();

    Ok(logs)
}

pub fn recent_with_activities(conn: &Connection, limit: i64) -> Result<Vec<LogWithActivity>> {
    let mut stmt = conn.prepare(
        "SELECT l.id, l.activity_id, a.name, l.starts_at, l.ends_at 
             FROM logs l 
             JOIN activities a ON l.activity_id = a.id 
             ORDER BY l.starts_at DESC LIMIT ?1",
    )?;

    let logs = stmt
        .query_map(rusqlite::params![limit], |row| {
            let activity_name = row
                .get::<_, Option<String>>(2)?
                .expect("Database corruption: missing activity name");
            let starts_at_str = row
                .get::<_, Option<String>>(3)?
                .expect("Database corruption: missing starts_at");
            let ends_at_str = row.get::<_, Option<String>>(4)?;

            Ok(LogWithActivity {
                id: row.get(0)?,
                activity_id: row.get(1)?,
                activity_name,
                starts_at: repository::parse_datetime(Some(starts_at_str))
                    .expect("Invalid starts_at datetime in database")
                    .unwrap(),
                ends_at: repository::parse_datetime(ends_at_str)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();

    Ok(logs)
}

pub fn update_start(conn: &Connection, id: Id, starts_at: DateTime<Utc>) -> Result<()> {
    conn.execute(
        "UPDATE logs SET starts_at = ?1 WHERE id = ?2",
        rusqlite::params![starts_at.format("%Y-%m-%d %H:%M:%S").to_string(), id],
    )?;
    Ok(())
}

pub fn update_end(conn: &Connection, id: Id, ends_at: Option<DateTime<Utc>>) -> Result<()> {
    conn.execute(
        "UPDATE logs SET ends_at = ?1 WHERE id = ?2",
        rusqlite::params![
            ends_at.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            id
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: Id) -> Result<()> {
    conn.execute("DELETE FROM logs WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
