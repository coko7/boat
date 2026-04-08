use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;

use boat_lib::models::activity::NewActivity;
use boat_lib::models::log::NewLog;
use boat_lib::repository::{activities_repository as activities, logs_repository as logs};

fn setup_conn() -> Connection {
    let conn = Connection::open_in_memory().expect("should create in-memory sqlite");
    let schema = include_str!("../src/schema.sql");
    conn.execute_batch(schema).expect("db schema should init");
    conn
}

fn make_test_activity(conn: &mut Connection) -> i64 {
    let act = activities::create(
        conn,
        NewActivity {
            name: "ForLogging".into(),
            description: None,
            tags: vec![],
        },
    )
    .expect("create activity");
    act.id
}

#[test]
fn create_and_get_log() -> Result<()> {
    let mut conn = setup_conn();
    let activity_id = make_test_activity(&mut conn);
    let now = Utc::now()
        .naive_utc()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let now_dt = chrono::NaiveDateTime::parse_from_str(&now, "%Y-%m-%d %H:%M:%S").unwrap();
    let now_dt_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(now_dt, Utc);
    let new_log = NewLog {
        activity_id,
        starts_at: now_dt_utc,
        ends_at: None,
    };
    let created = logs::create(&conn, new_log)?;
    let fetched = logs::get_by_id(&conn, created.id)?;
    assert_eq!(fetched.activity_id, activity_id);
    assert_eq!(fetched.starts_at.timestamp(), now_dt_utc.timestamp());
    assert!(fetched.ends_at.is_none());
    Ok(())
}

#[test]
fn log_start_update() -> Result<()> {
    let mut conn = setup_conn();
    let activity_id = make_test_activity(&mut conn);
    let now = chrono::NaiveDate::from_ymd_opt(2022, 2, 1)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let now_dt_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(now, Utc);
    let created = logs::create(
        &conn,
        NewLog {
            activity_id,
            starts_at: now_dt_utc,
            ends_at: None,
        },
    )?;
    let new_start_time = chrono::NaiveDate::from_ymd_opt(2022, 2, 1)
        .unwrap()
        .and_hms_opt(11, 0, 0)
        .unwrap();
    let new_start_time_utc =
        chrono::DateTime::<Utc>::from_naive_utc_and_offset(new_start_time, Utc);
    logs::update_start(&conn, created.id, new_start_time_utc)?;
    let updated = logs::get_by_id(&conn, created.id)?;
    assert_eq!(
        updated.starts_at.timestamp(),
        new_start_time_utc.timestamp()
    );
    Ok(())
}

#[test]
fn log_end_update() -> Result<()> {
    let mut conn = setup_conn();
    let activity_id = make_test_activity(&mut conn);
    let now = chrono::NaiveDate::from_ymd_opt(2022, 3, 1)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let now_dt_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(now, Utc);
    let created = logs::create(
        &conn,
        NewLog {
            activity_id,
            starts_at: now_dt_utc,
            ends_at: None,
        },
    )?;
    let end_time = chrono::NaiveDate::from_ymd_opt(2022, 3, 1)
        .unwrap()
        .and_hms_opt(13, 0, 0)
        .unwrap();
    let end_time_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(end_time, Utc);
    logs::update_end(&conn, created.id, Some(end_time_utc))?;
    let ended = logs::get_by_id(&conn, created.id)?;
    assert_eq!(ended.ends_at.unwrap().timestamp(), end_time_utc.timestamp());
    Ok(())
}

#[test]
fn get_for_activity_and_recent_with_activities() -> Result<()> {
    let mut conn = setup_conn();
    let activity_id = make_test_activity(&mut conn);
    // Insert multiple logs with different start times
    for offset_minutes in 0..3 {
        let base = chrono::NaiveDate::from_ymd_opt(2022, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let dt = base + chrono::Duration::minutes(offset_minutes);
        let log = NewLog {
            activity_id,
            starts_at: chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc),
            ends_at: None,
        };
        logs::create(&conn, log)?;
    }
    let for_activity = logs::get_for_activity(&conn, activity_id)?;
    assert_eq!(for_activity.len(), 3);
    // Exercise recent_with_activities:
    let recent = logs::recent_with_activities(&conn, 5)?;
    assert!(recent.len() >= 3);
    Ok(())
}

#[test]
fn delete_log_works() -> Result<()> {
    let mut conn = setup_conn();
    let activity_id = make_test_activity(&mut conn);
    let now = chrono::NaiveDate::from_ymd_opt(2022, 2, 2)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let now_dt_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(now, Utc);
    let log = logs::create(
        &conn,
        NewLog {
            activity_id,
            starts_at: now_dt_utc,
            ends_at: None,
        },
    )?;
    logs::delete(&conn, log.id)?;
    assert!(logs::get_by_id(&conn, log.id).is_err());
    Ok(())
}
