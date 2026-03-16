use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use log::debug;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead},
};

pub type ActivityId = u64;

#[derive(Debug)]
pub struct ActivityStore {
    activities: HashMap<ActivityId, Activity>,
}

// impl ActivityStore {
//     fn update_activity(&mut self, id: ActivityId) {}
// }

#[derive(Debug)]
pub struct Activity {
    id: ActivityId,
    name: String,
    category: String,
    tracking: HashSet<(DateTime<Local>, Option<DateTime<Local>>)>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ActivityRecord {
    name: String,
    category: Option<String>,
    start: DateTime<Local>,
    end: Option<DateTime<Local>>,
}

pub fn load_from_file() -> Result<Vec<Activity>> {
    let file = File::open("data/activities.txt")?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let act_record = parse_activity_record_line(&line);
        println!("{}", line);
    }

    todo!()
}

fn parse_activity_record_line(line: &str) -> Result<ActivityRecord> {
    let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
    debug!("parts: {parts:?}");

    let (start, end) = parts
        .first()
        .context("expects act times at 0")?
        .split_once('>')
        .context("should be two")?;

    let start = start.trim();
    let end = end.trim();

    debug!("raw start time: {}", start);
    debug!("raw end time: {} ({} chars)", end, end.len());

    let start = parse_local_dt(start)?;
    debug!("parsed start time: {}", start);

    let end = (!end.is_empty()).then(|| parse_local_dt(end)).transpose()?;

    debug!("parsed end time: {end:?}");

    let category = parts
        .get(1)
        .context("expects category part at 2")?
        .to_string();
    let category = (!category.is_empty()).then_some(category);
    debug!("parsed catogery name: {category:?}");

    let name = parts
        .get(2)
        .context("expects activity name at 3")?
        .to_string();
    debug!("parsed activity name: {name}");

    let act = ActivityRecord {
        name,
        category,
        start,
        end,
    };
    Ok(act)
}

/// Parses naive datetime string to local timezone.
/// Handles DST gaps/folds by picking earliest valid.
pub fn parse_local_dt(s: &str) -> Result<DateTime<Local>> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
        .with_context(|| format!("Failed to parse '{s}' as naive datetime"))?;

    Local
        .from_local_datetime(&naive)
        .earliest()
        .context("Invalid local time (e.g., DST gap)")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_logger() {
        use env_logger::Builder;
        let _ = Builder::from_default_env()
            .is_test(true) // CRUCIAL: enables test capture
            .filter_level(log::LevelFilter::Trace)
            .try_init();
    }

    #[test]
    fn test_parse_local_dt_err() {
        setup_logger();
        let result = parse_local_dt("invalid");
        assert!(result.is_err());
        // assert_eq!(
        //     result.unwrap_err().to_string(),
        //     "Failed to parse datetime string: input has length 7, but expected pattern has length at least 19"
        // );
    }

    #[test]
    fn test_parse_local_dt_ok() {
        setup_logger();
        let fmt = "%Y-%m-%d %H:%M:%S";
        let exp = parse_local_dt("2023-10-05 14:30:00").unwrap();

        assert_eq!(exp.format(fmt).to_string(), format!("2023-10-05 14:30:00"));
    }

    #[test]
    fn test_parse_activity_record() {
        setup_logger();
        let exp = ActivityRecord {
            name: "do some stuff".to_string(),
            category: Some("misc".to_string()),
            start: parse_local_dt("2026-03-16 08:00:00").unwrap(),
            end: Some(parse_local_dt("2026-03-16 09:30:00").unwrap()),
        };
        let from_file = "2026-03-16 08:00:00 > 2026-03-16 09:30:00 | misc | do some stuff";
        let actual = parse_activity_record_line(from_file).unwrap();
        assert_eq!(exp, actual)
    }

    #[test]
    fn test_parse_activity_record_ongoing() {
        setup_logger();
        let exp = ActivityRecord {
            name: "do some stuff".to_string(),
            category: Some("misc".to_string()),
            start: parse_local_dt("2026-03-16 08:00:00").unwrap(),
            end: None,
        };
        let from_file = "2026-03-16 08:00:00 > | misc | do some stuff";
        let actual = parse_activity_record_line(from_file).unwrap();
        assert_eq!(exp, actual)
    }

    #[test]
    fn test_parse_activity_record_no_category() {
        setup_logger();
        let exp = ActivityRecord {
            name: "do some stuff".to_string(),
            category: None,
            start: parse_local_dt("2026-03-16 08:00:00").unwrap(),
            end: None,
        };
        let from_file = "2026-03-16 08:00:00 > | | do some stuff";
        let actual = parse_activity_record_line(from_file).unwrap();
        assert_eq!(exp, actual)
    }
}
