use anyhow::Result;
use rusqlite::Connection;

use boat_lib::models::activity::NewActivity;
use boat_lib::repository::activities_repository as activities;
use boat_lib::repository::logs_repository as logs;
use boat_lib::repository::tags_repository as tags;

fn setup_conn() -> Connection {
    let conn = Connection::open_in_memory().expect("should create in-memory sqlite");
    let schema = include_str!("../src/schema.sql");
    conn.execute_batch(schema).expect("db schema should init");
    conn
}

#[test]
fn create_and_fetch_activity_with_tags() -> Result<()> {
    let mut conn = setup_conn();
    let new_activity = NewActivity {
        name: "Read book".to_string(),
        description: Some("Leisure time".to_string()),
        tags: vec!["relax".to_string(), "reading".to_string()],
    };
    let created = activities::create(&mut conn, new_activity)?;
    let fetched = activities::get_by_id(&conn, created.id)?;

    assert_eq!(fetched.name, "Read book");
    assert_eq!(fetched.description, Some("Leisure time".to_string()));
    assert_eq!(fetched.tags.len(), 2);
    assert!(fetched.tags.contains("relax"));
    assert!(fetched.tags.contains("reading"));
    Ok(())
}

#[test]
fn update_activity_and_tags() -> Result<()> {
    let mut conn = setup_conn();
    let act = activities::create(
        &mut conn,
        NewActivity {
            name: "Original".to_string(),
            description: Some("desc".to_string()),
            tags: vec!["a".to_string(), "b".to_string()],
        },
    )?;
    activities::update(
        &mut conn,
        act.id,
        Some("Updated!"),
        Some("new desc"),
        Some(&["b".to_string(), "c".to_string()]),
    )?;
    let fetched = activities::get_by_id(&conn, act.id)?;
    assert_eq!(fetched.name, "Updated!");
    assert_eq!(fetched.description, Some("new desc".to_string()));
    // Should replace tags
    assert_eq!(fetched.tags.len(), 2);
    assert!(fetched.tags.contains("b"));
    assert!(fetched.tags.contains("c"));
    Ok(())
}

#[test]
fn delete_activity_removes_tags_link() -> Result<()> {
    let mut conn = setup_conn();
    let act = activities::create(
        &mut conn,
        NewActivity {
            name: "ToDelete".to_string(),
            description: None,
            tags: vec!["todelete".to_string()],
        },
    )?;
    let tag_before = tags::get_by_name(&conn, "todelete")?;
    assert!(tag_before.is_some());
    activities::delete(&conn, act.id)?;
    // Tag still exists, but relation is gone
    let tag = tags::get_by_name(&conn, "todelete")?;
    assert!(tag.is_some());
    // Activity is gone
    let a = activities::get_by_id(&conn, act.id);
    assert!(a.is_err());
    Ok(())
}

#[test]
fn get_all_activities_lists_inserted() -> Result<()> {
    let mut conn = setup_conn();
    let _ = activities::create(
        &mut conn,
        NewActivity {
            name: "Walk".to_string(),
            description: None,
            tags: vec![],
        },
    )?;
    let _ = activities::create(
        &mut conn,
        NewActivity {
            name: "Run".to_string(),
            description: None,
            tags: vec!["sport".to_string()],
        },
    )?;
    let all = activities::get_all(&conn)?;
    let names: Vec<_> = all.iter().map(|a| a.name.as_str()).collect();
    assert!(names.contains(&"Walk"));
    assert!(names.contains(&"Run"));
    Ok(())
}

#[test]
fn start_stop_and_get_current_ongoing() -> Result<()> {
    let mut conn = setup_conn();

    // Create two activities
    let a1 = activities::create(
        &mut conn,
        NewActivity {
            name: "First".to_string(),
            description: None,
            tags: vec![],
        },
    )?;
    let a2 = activities::create(
        &mut conn,
        NewActivity {
            name: "Second".to_string(),
            description: None,
            tags: vec![],
        },
    )?;

    // No activity started yet
    assert!(activities::get_current_ongoing(&conn)?.is_none());

    // Start first activity
    activities::start(&mut conn, a1.id)?;
    let current = activities::get_current_ongoing(&conn)?;
    assert!(current.is_some());
    assert_eq!(current.as_ref().unwrap().id, a1.id);

    // Start second activity (should stop the first, and start the second)
    activities::start(&mut conn, a2.id)?;
    let current = activities::get_current_ongoing(&conn)?;
    assert!(current.is_some());
    assert_eq!(current.as_ref().unwrap().id, a2.id);

    // Stopping should make none ongoing
    activities::stop_current(&conn)?;
    assert!(activities::get_current_ongoing(&conn)?.is_none());
    Ok(())
}

#[test]
fn idempotent_start_activity_should_not_create_extra_log() -> Result<()> {
    let mut conn = setup_conn();

    // Create activity
    let a1 = activities::create(
        &mut conn,
        NewActivity {
            name: "A".to_string(),
            description: None,
            tags: vec![],
        },
    )?;

    // Start it once
    activities::start(&mut conn, a1.id)?;
    let logs_before = logs::get_for_activity(&conn, a1.id)?;
    assert_eq!(logs_before.len(), 1);
    let log_id = logs_before[0].id;

    // Start it again (should not create a new log)
    activities::start(&mut conn, a1.id)?;
    let logs_after = logs::get_for_activity(&conn, a1.id)?;
    assert_eq!(
        logs_after.len(),
        1,
        "second start should not create new log"
    );
    assert_eq!(logs_after[0].id, log_id, "should be the same log record");
    assert_eq!(logs_after[0].ends_at, None, "log should still be ongoing");
    Ok(())
}
