use chrono::{DateTime, Datelike, Duration, Local, NaiveDate};

pub fn pretty_format_duration(mut seconds: i64, long_format: bool) -> String {
    let hours = seconds / 3600;
    seconds %= 3600;

    let minutes = seconds / 60;
    seconds %= 60;

    let mut parts = Vec::new();

    if hours > 0 {
        let suffix = if long_format { " hours" } else { "h" }.to_string();
        parts.push(format!("{hours}{suffix}"));
    }

    if minutes > 0 {
        let suffix = if long_format { " minutes" } else { "m" }.to_string();
        parts.push(format!("{minutes}{suffix}"));
    }

    if hours == 0 && minutes == 0 {
        let suffix = if long_format { " seconds" } else { "s" }.to_string();
        parts.push(format!("{seconds}{suffix}"));
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pretty_format_duration() {
        assert_eq!(&pretty_format_duration(0, false), "0s");
        assert_eq!(&pretty_format_duration(42, false), "42s");
        assert_eq!(&pretty_format_duration(60, false), "1m");
        assert_eq!(&pretty_format_duration(61, false), "1m");
        assert_eq!(&pretty_format_duration(3600, false), "1h");
        assert_eq!(&pretty_format_duration(3601, false), "1h");
        assert_eq!(&pretty_format_duration(3661, false), "1h 1m");
    }
}

pub enum DateTimeRenderMode {
    TimeOnly,
    DateOnly,
    DateAndTime,
}

impl DateTimeRenderMode {
    pub fn render_naive_date(&self, dt: &chrono::NaiveDate) -> String {
        let fmt = match self {
            DateTimeRenderMode::TimeOnly => "%H:%M",
            DateTimeRenderMode::DateOnly => "%Y-%m-%d",
            DateTimeRenderMode::DateAndTime => "%Y-%m-%d %H:%M",
        };

        dt.format(fmt).to_string()
    }
    pub fn render_date_time<Tz>(&self, dt: chrono::DateTime<Tz>) -> String
    where
        Tz: chrono::TimeZone,
        Tz::Offset: std::fmt::Display,
    {
        let fmt = match self {
            DateTimeRenderMode::TimeOnly => "%H:%M",
            DateTimeRenderMode::DateOnly => "%Y-%m-%d",
            DateTimeRenderMode::DateAndTime => "%Y-%m-%d %H:%M",
        };

        dt.format(fmt).to_string()
    }
}

pub fn is_today<Tz>(dt: DateTime<Tz>) -> bool
where
    Tz: chrono::TimeZone,
{
    let today = Local::now().date_naive();
    let other_dt = dt.with_timezone(&Local);
    other_dt.date_naive() == today
}

pub fn is_yesterday<Tz>(dt: DateTime<Tz>) -> bool
where
    Tz: chrono::TimeZone,
{
    let today = Local::now().date_naive();
    let yesterday = today - Duration::days(1);
    let other_dt = dt.with_timezone(&Local);
    other_dt.date_naive() == yesterday
}

pub fn is_this_week<Tz>(dt: DateTime<Tz>) -> bool
where
    Tz: chrono::TimeZone,
{
    let today = Local::now().date_naive();
    let this_week_iso = today.iso_week();

    let other_dt = dt.with_timezone(&Local);
    let other_week_iso = other_dt.date_naive().iso_week();

    this_week_iso.year() == other_week_iso.year() && this_week_iso.week() == other_week_iso.week()
}

pub fn is_last_week<Tz>(dt: DateTime<Tz>) -> bool
where
    Tz: chrono::TimeZone,
{
    let today = Local::now().date_naive();
    let other_dt = dt.with_timezone(&Local);

    let this_week_iso = today.iso_week();
    let this_week_year = this_week_iso.year();
    let this_week = this_week_iso.week();

    let other_week_iso = other_dt.date_naive().iso_week();
    let other_week_year = other_week_iso.year();
    let other_week = other_week_iso.week();

    if this_week > 1 {
        other_week_year == this_week_year && other_week == this_week - 1
    } else {
        let last_year = this_week_year - 1;

        // Find previous ISO week by handling year boundary using the December 28 trick:
        // - ISO weeks start with Monday and end on Sunday.
        // - Each week's year is the Gregorian year in which the Thursday falls.
        // - The first week of the year, hence, always contains January 4.
        // - This means 7 days before is guaranteed to be the previous year.
        // - 7 days before Jan 4 gives us Dec 28.
        // Learn more: https://en.wikipedia.org/wiki/ISO_week_date
        let last_week_of_last_year = NaiveDate::from_ymd_opt(last_year, 12, 28)
            .unwrap()
            .iso_week()
            .week();
        other_week_year == last_year && other_week == last_week_of_last_year
    }
}

pub fn is_this_month<Tz>(dt: DateTime<Tz>) -> bool
where
    Tz: chrono::TimeZone,
{
    let today = Local::now().date_naive();
    let other_dt = dt.with_timezone(&Local);

    let this_year = today.year();
    let this_month = today.month();

    let other = other_dt.date_naive();
    other.year() == this_year && other.month() == this_month
}

pub fn is_last_month<Tz>(dt: DateTime<Tz>) -> bool
where
    Tz: chrono::TimeZone,
{
    let today = Local::now().date_naive();
    let other_dt = dt.with_timezone(&Local);

    let this_year = today.year();
    let this_month = today.month();

    let (last_month_year, last_month) = if this_month > 1 {
        (this_year, this_month - 1)
    } else {
        (this_year - 1, 12)
    };

    let other = other_dt.date_naive();
    other.year() == last_month_year && other.month() == last_month
}

pub fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| format!("invalid date '{s}', expected format YYYY-MM-DD"))
}

#[cfg(test)]
mod parse_date_tests {
    use super::*;

    #[test]
    fn parse_date_valid_should_succeed() {
        assert_eq!(
            parse_date("2023-08-14").unwrap(),
            NaiveDate::from_ymd_opt(2023, 8, 14).unwrap()
        );
    }
    #[test]
    fn parse_date_fails_invalid_should_fail() {
        let e = parse_date("nope").unwrap_err();
        assert!(e.contains("invalid date"));
    }
}

#[cfg(test)]
mod date_check_tests {
    use super::*;
    use chrono::{Local, Months};

    // --- pretty_format_duration (long format) ---

    #[test]
    fn pretty_format_duration_long_format_seconds() {
        assert_eq!(pretty_format_duration(0, true), "0 seconds");
        assert_eq!(pretty_format_duration(45, true), "45 seconds");
    }

    #[test]
    fn pretty_format_duration_long_format_minutes() {
        assert_eq!(pretty_format_duration(60, true), "1 minutes");
        assert_eq!(pretty_format_duration(90, true), "1 minutes");
    }

    #[test]
    fn pretty_format_duration_long_format_hours_and_minutes() {
        assert_eq!(pretty_format_duration(3661, true), "1 hours 1 minutes");
    }

    // --- DateTimeRenderMode ---

    #[test]
    fn render_naive_date_date_only() {
        let d = NaiveDate::from_ymd_opt(2024, 4, 15).unwrap();
        assert_eq!(
            DateTimeRenderMode::DateOnly.render_naive_date(&d),
            "2024-04-15"
        );
    }

    #[test]
    fn render_date_time_date_only() {
        use chrono::{TimeZone, Utc};
        let dt = Utc.with_ymd_and_hms(2024, 4, 15, 13, 30, 0).unwrap();
        assert_eq!(
            DateTimeRenderMode::DateOnly.render_date_time(dt),
            "2024-04-15"
        );
    }

    #[test]
    fn render_date_time_time_only() {
        use chrono::{TimeZone, Utc};
        let dt = Utc.with_ymd_and_hms(2024, 4, 15, 13, 30, 0).unwrap();
        assert_eq!(DateTimeRenderMode::TimeOnly.render_date_time(dt), "13:30");
    }

    #[test]
    fn render_date_time_date_and_time() {
        use chrono::{TimeZone, Utc};
        let dt = Utc.with_ymd_and_hms(2024, 4, 15, 13, 30, 0).unwrap();
        assert_eq!(
            DateTimeRenderMode::DateAndTime.render_date_time(dt),
            "2024-04-15 13:30"
        );
    }

    // --- is_today / is_yesterday ---

    #[test]
    fn is_today_with_now() {
        assert!(is_today(Local::now()));
    }

    #[test]
    fn is_today_with_yesterday() {
        assert!(!is_today(Local::now() - Duration::days(1)));
    }

    #[test]
    fn is_yesterday_with_one_day_ago() {
        assert!(is_yesterday(Local::now() - Duration::days(1)));
    }

    #[test]
    fn is_yesterday_with_today() {
        assert!(!is_yesterday(Local::now()));
    }

    #[test]
    fn is_yesterday_with_two_days_ago() {
        assert!(!is_yesterday(Local::now() - Duration::days(2)));
    }

    // --- is_this_week / is_last_week ---

    #[test]
    fn is_this_week_with_now() {
        assert!(is_this_week(Local::now()));
    }

    #[test]
    fn is_this_week_with_seven_days_ago() {
        // 7 days ago is always the previous ISO week
        assert!(!is_this_week(Local::now() - Duration::days(7)));
    }

    #[test]
    fn is_last_week_with_seven_days_ago() {
        // Subtracting exactly 7 days always lands in the previous ISO week
        assert!(is_last_week(Local::now() - Duration::days(7)));
    }

    #[test]
    fn is_last_week_with_now() {
        assert!(!is_last_week(Local::now()));
    }

    #[test]
    fn is_last_week_with_fourteen_days_ago() {
        assert!(!is_last_week(Local::now() - Duration::days(14)));
    }

    // --- is_this_month / is_last_month ---

    #[test]
    fn is_this_month_with_now() {
        assert!(is_this_month(Local::now()));
    }

    #[test]
    fn is_this_month_with_forty_days_ago() {
        assert!(!is_this_month(Local::now() - Duration::days(40)));
    }

    #[test]
    fn is_last_month_with_last_month() {
        assert!(is_last_month(Local::now() - Months::new(1)));
    }

    #[test]
    fn is_last_month_with_now() {
        assert!(!is_last_month(Local::now()));
    }

    #[test]
    fn is_last_month_with_two_months_ago() {
        assert!(!is_last_month(Local::now() - Months::new(2)));
    }
}
