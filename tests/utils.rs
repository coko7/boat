use anyhow::Result;
use chrono::{Datelike, Timelike};

use boat_lib::utils::{init_database, parse_local_dt};

#[test]
fn parse_local_dt_parses_correctly() -> Result<()> {
    let dt = parse_local_dt("2023-01-02 15:04:05")?;
    assert_eq!(dt.year(), 2023);
    assert_eq!(dt.month(), 1);
    assert_eq!(dt.day(), 2);

    assert_eq!(dt.hour(), 15);
    assert_eq!(dt.minute(), 4);
    assert_eq!(dt.second(), 5);
    Ok(())
}

#[test]
fn init_database_inserts_schema() -> Result<()> {
    let tmp = tempfile::NamedTempFile::new()?;
    let conn = init_database(tmp.path())?;
    // Should create tags table
    let mut stmt =
        conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='tags'")?;
    let mut rows = stmt.query([])?;
    assert!(rows.next()?.is_some());
    Ok(())
}
