use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

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

#[cfg(test)]
pub fn init_test_logger() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();
}

#[cfg(test)]
mod tests {
    use chrono::Timelike;

    use super::*;

    #[test]
    fn test_parse_local_dt_with_valid_datetime_should_be_ok() -> Result<()> {
        init_test_logger();
        let dt = parse_local_dt("2023-10-01 12:34:56")?;
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2023-10-01 12:34:56"
        );
        Ok(())
    }

    #[test]
    fn test_parse_local_dt_with_fractional_seconds_should_be_ok() -> Result<()> {
        init_test_logger();
        let dt = parse_local_dt("2023-10-01 12:34:56.789")?;
        assert_eq!(dt.timestamp_nanos_opt(), Some(1696156496789000000));
        Ok(())
    }

    #[test]
    fn test_parse_local_dt_dst_fallback_ambiguous_earliest_should_be_ok() -> Result<()> {
        init_test_logger();
        // Europe/Paris DST fallback 2023-10-29 03:00 occurs twice
        let naive =
            NaiveDateTime::parse_from_str("2023-10-29 03:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let s = naive.format("%Y-%m-%d %H:%M:%S").to_string();

        let dt = parse_local_dt(&s)?;
        // .earliest() picks the first occurrence (DST, later UTC offset)
        assert_eq!(dt.hour(), 3);
        Ok(())
    }

    #[test]
    fn test_parse_local_dt_non_dst_normal_time_should_be_ok() -> Result<()> {
        init_test_logger();
        let dt = parse_local_dt("2023-01-15 14:20:30")?;
        // Should parse without issues outside DST transitions
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2023-01-15 14:20:30"
        );
        Ok(())
    }

    #[test]
    fn test_parse_local_dt_edge_midnight_should_be_ok() -> Result<()> {
        init_test_logger();
        let dt = parse_local_dt("2024-01-01 00:00:00")?;
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        Ok(())
    }

    #[test]
    fn test_parse_local_dt_edge_noon_should_be_ok() -> Result<()> {
        init_test_logger();
        let dt = parse_local_dt("2024-07-01 12:00:00")?;
        assert_eq!(dt.hour(), 12);
        Ok(())
    }

    #[test]
    fn test_parse_local_dt_with_invalid_format_should_be_err() {
        init_test_logger();
        let err = parse_local_dt("2023-10-01").expect_err("Should fail on invalid format");
        assert!(
            err.to_string()
                .contains("failed to parse '2023-10-01' as naive datetime")
        );
    }

    #[test]
    fn test_parse_local_dt_with_invalid_argument_should_be_err() -> Result<()> {
        init_test_logger();
        let err = parse_local_dt("invalid").expect_err("Should fail on invalid format");
        assert!(
            err.to_string()
                .contains("failed to parse 'invalid' as naive datetime")
        );
        Ok(())
    }

    #[test]
    fn test_parse_local_dt_dst_gap_invalid_time_should_be_err() {
        init_test_logger();
        // Example DST gap: Europe/Paris 2023-03-26 02:00-03:00 doesn't exist (clocks jump to 03:00)
        // Use 02:30 as invalid local time
        let naive =
            NaiveDateTime::parse_from_str("2023-03-26 02:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let s = naive.format("%Y-%m-%d %H:%M:%S").to_string();

        let err = parse_local_dt(&s).expect_err("Should fail on DST gap");
        assert!(
            err.to_string()
                .contains("invalid local time (e.g., DST gap)")
        );
    }
}
