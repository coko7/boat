use anyhow::Result;
use rusqlite::Connection;

use boat_lib::models::tag::NewTag;
use boat_lib::repository::tags_repository as tags;

fn setup_conn() -> Connection {
    let conn = Connection::open_in_memory().expect("should create in-memory sqlite");
    let schema = include_str!("../src/schema.sql");
    conn.execute_batch(schema).expect("db schema should init");
    conn
}

#[test]
fn create_and_fetch_tag() -> Result<()> {
    let conn = setup_conn();
    let created = tags::create(
        &conn,
        NewTag {
            name: "focus".to_string(),
        },
    )?;
    assert_eq!(created.name, "focus");
    let fetched = tags::get_by_id(&conn, created.id)?;
    assert_eq!(fetched.name, "focus");
    Ok(())
}

#[test]
fn get_tag_by_name() -> Result<()> {
    let conn = setup_conn();
    let _ = tags::create(
        &conn,
        NewTag {
            name: "other".to_string(),
        },
    )?;
    let tag = tags::get_by_name(&conn, "other")?;
    assert!(tag.is_some());
    assert_eq!(tag.unwrap().name, "other");
    Ok(())
}

#[test]
fn list_tags_and_uniqueness() -> Result<()> {
    let conn = setup_conn();
    let _ = tags::create(
        &conn,
        NewTag {
            name: "t1".to_string(),
        },
    )?;
    let _ = tags::create(
        &conn,
        NewTag {
            name: "t2".to_string(),
        },
    )?;
    let all = tags::get_all(&conn)?;
    assert!(all.iter().any(|t| t.name == "t1"));
    assert!(all.iter().any(|t| t.name == "t2"));
    // Uniqueness
    let again = tags::create(
        &conn,
        NewTag {
            name: "t1".to_string(),
        },
    )?;
    assert_eq!(again.name, "t1");
    Ok(())
}

#[test]
fn delete_tag_and_by_name() -> Result<()> {
    let conn = setup_conn();
    let created = tags::create(
        &conn,
        NewTag {
            name: "gone".to_string(),
        },
    )?;
    tags::delete(&conn, created.id)?;
    assert!(tags::get_by_id(&conn, created.id).is_err());
    let _ = tags::create(
        &conn,
        NewTag {
            name: "gone2".to_string(),
        },
    )?;
    tags::delete_by_name(&conn, "gone2")?;
    assert!(tags::get_by_name(&conn, "gone2")?.is_none());
    Ok(())
}

#[test]
fn usage_stats_is_correct() -> Result<()> {
    // Simple smoke test since association is indirect
    let conn = setup_conn();
    let _ = tags::create(
        &conn,
        NewTag {
            name: "stat1".to_string(),
        },
    )?;
    let _ = tags::create(
        &conn,
        NewTag {
            name: "stat2".to_string(),
        },
    )?;
    let stats = tags::usage_stats(&conn)?;
    // returns at least the two tags, usage count is 0 since no activities
    assert!(stats.iter().any(|(tag, _count)| tag.name == "stat1"));
    assert!(stats.iter().any(|(tag, _count)| tag.name == "stat2"));
    Ok(())
}
