use log::debug;
use rusqlite::{Connection, OptionalExtension, Result};
use std::collections::HashSet;

use crate::{
    models::activity::{Activity, ActivityBase, NewActivity},
    repository::{Id, logs_repository},
};

pub fn create(conn: &mut Connection, new_activity: NewActivity) -> Result<Activity> {
    let tx = conn.transaction()?;

    // Insert activity
    tx.execute(
        "INSERT INTO activities (name, description) VALUES (?1, ?2)",
        rusqlite::params![new_activity.name, new_activity.description],
    )?;
    let activity_id = tx.last_insert_rowid();

    // Insert or reuse tags, then link them
    for tag_name in new_activity.tags {
        // Upsert tag
        tx.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            rusqlite::params![tag_name],
        )?;

        let tag_id: i64 = tx.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            rusqlite::params![tag_name],
            |row| row.get(0),
        )?;

        // Link activity to tag
        tx.execute(
            "INSERT OR IGNORE INTO activities_tags (activity_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![activity_id, tag_id],
        )?;
    }

    tx.commit()?;
    get_by_id(conn, activity_id)
}

pub fn get_by_id(conn: &Connection, id: Id) -> Result<Activity> {
    let base = get_base(conn, id)?;
    let tags = load_tags(conn, id)?;
    debug!("tags for activity {id}: {tags:?}");
    let logs = logs_repository::get_for_activity(conn, id)?;
    debug!("logs for activity {id}: {logs:?}");

    Ok(Activity {
        id: base.id,
        name: base.name,
        description: base.description,
        tags,
        logs,
    })
}

fn get_base(conn: &Connection, id: Id) -> Result<ActivityBase> {
    conn.query_row(
        "SELECT id, name, description FROM activities WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(ActivityBase {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        },
    )
}

fn load_tags(conn: &Connection, activity_id: Id) -> Result<HashSet<String>> {
    let mut tags = HashSet::new();
    let mut stmt = conn.prepare(
        "SELECT t.name FROM activities_tags at
             JOIN tags t ON at.tag_id = t.id
             WHERE at.activity_id = ?1",
    )?;
    let rows = stmt.query_map(rusqlite::params![activity_id], |row| row.get(0))?;
    for row in rows.filter_map(Result::ok) {
        tags.insert(row);
    }
    Ok(tags)
}

pub fn get_all(conn: &Connection) -> Result<Vec<Activity>> {
    let mut stmt = conn.prepare("SELECT id, name, description FROM activities ORDER BY name")?;
    let activities = stmt
        .query_map([], |row| {
            Ok(ActivityBase {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?
        .filter_map(Result::ok)
        .map(|base| get_by_id(conn, base.id))
        .collect::<Result<Vec<_>>>()?;

    Ok(activities)
}

pub fn update(
    conn: &mut Connection,
    id: Id,
    name: Option<&str>,
    description: Option<&str>,
    tags: Option<&[String]>,
) -> Result<()> {
    let tx = conn.transaction()?;

    if let Some(name) = name {
        tx.execute(
            "UPDATE activities SET name = ?1 WHERE id = ?2",
            rusqlite::params![name, id],
        )?;
    }

    if let Some(description) = description {
        tx.execute(
            "UPDATE activities SET description = ?1 WHERE id = ?2",
            rusqlite::params![description, id],
        )?;
    }

    if let Some(new_tags) = tags {
        // Delete existing links
        tx.execute(
            "DELETE FROM activities_tags WHERE activity_id = ?1",
            rusqlite::params![id],
        )?;

        for tag_name in new_tags {
            tx.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                rusqlite::params![tag_name],
            )?;

            let tag_id: Id = tx.query_row(
                "SELECT id FROM tags WHERE name = ?1",
                rusqlite::params![tag_name],
                |row| row.get(0),
            )?;

            tx.execute(
                "INSERT INTO activities_tags (activity_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![id, tag_id],
            )?;
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn delete(conn: &Connection, id: Id) -> Result<()> {
    conn.execute(
        "DELETE FROM activities WHERE id = ?1",
        rusqlite::params![id],
    )?;
    Ok(())
}

pub fn get_current_ongoing(conn: &Connection) -> Result<Option<Activity>> {
    let activity_id_opt: Option<i64> = conn
        .query_row(
            "SELECT activity_id FROM logs WHERE ends_at IS NULL LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;

    activity_id_opt.map(|id| get_by_id(conn, id)).transpose()
}

pub fn start(conn: &mut Connection, activity_id: Id) -> Result<()> {
    let tx = conn.transaction()?;

    debug!("stopping any ongoing activity first");
    // End any existing ongoing log
    tx.execute(
        "UPDATE logs SET ends_at = CURRENT_TIMESTAMP WHERE ends_at IS NULL",
        [],
    )?;

    debug!("creating a new log for activity: {activity_id}");
    tx.execute(
        "INSERT INTO logs (activity_id, starts_at) VALUES (?1, CURRENT_TIMESTAMP)",
        rusqlite::params![activity_id],
    )?;

    tx.commit()?;
    Ok(())
}

pub fn stop_current(conn: &Connection) -> Result<()> {
    conn.execute(
        "UPDATE logs SET ends_at = CURRENT_TIMESTAMP WHERE ends_at IS NULL",
        [],
    )?;
    Ok(())
}
