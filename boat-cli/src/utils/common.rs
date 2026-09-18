use anyhow::Result;
use boat_lib::models::log::Log as DatabaseLog;
use chrono::{Local, NaiveDate};
use dialoguer::Confirm;
use log::debug;
use serde::Serialize;
use std::collections::HashSet;
use yansi::Paint;

use crate::{
    cli::{PeriodInput, PresetPeriod},
    models::{RowPrintable, TablePrintable},
    utils,
};

pub fn resolve_tri_state(a: bool, b: bool, c: bool) -> bool {
    match (a, b) {
        (true, false) => true,
        (false, true) => false,
        _ => c, // neither specified → fallback
    }
}

pub fn prompt_for_confirmation(msg: &str, default_value: bool) -> Result<bool> {
    let proceed = Confirm::new()
        .with_prompt(Paint::yellow(&msg).to_string())
        .default(default_value)
        .interact()?;

    Ok(proceed)
}

pub fn list_printable_items<T: RowPrintable + Serialize>(
    items: &Vec<T>,
    show_as_json: bool,
    fields: Option<&[String]>,
) -> Result<()> {
    if show_as_json {
        let json = serde_json::to_string(&items)?;
        println!("{json}");
        return Ok(());
    }

    if items.is_empty() {
        println!("no available data");
        return Ok(());
    }

    let table = items.to_printable_table(fields)?;
    println!("{table}");
    Ok(())
}

pub fn tags_str(tags: &HashSet<String>) -> String {
    let mut tags: Vec<_> = tags.iter().map(String::as_str).collect();
    tags.sort_unstable();
    tags.join(",")
}

pub fn matches_period_filter(log: &DatabaseLog, period_input: &PeriodInput) -> bool {
    match period_input {
        PeriodInput::Preset(preset_period) => matches_period(log, preset_period),
        PeriodInput::Single(date) => matches_date(log, date),
        PeriodInput::Range { start, end } => matches_date_range(log, start, end),
    }
}

pub fn matches_period(log: &DatabaseLog, period: &PresetPeriod) -> bool {
    debug!("checking if {log:?} matches the given period: {period:?}");

    match period {
        PresetPeriod::Today => utils::date::is_today(log.starts_at),
        PresetPeriod::Yesterday => utils::date::is_yesterday(log.starts_at),
        PresetPeriod::ThisWeek => utils::date::is_this_week(log.starts_at),
        PresetPeriod::LastWeek => utils::date::is_last_week(log.starts_at),
        PresetPeriod::ThisMonth => utils::date::is_this_month(log.starts_at),
        PresetPeriod::LastMonth => utils::date::is_last_month(log.starts_at),
        PresetPeriod::AllTime => true,
    }
}

pub fn matches_date(log: &DatabaseLog, date: &NaiveDate) -> bool {
    log.starts_at.date_naive() == *date
        && log.ends_at.unwrap_or(Local::now().into()).date_naive() == *date
}

pub fn matches_date_range(
    log: &DatabaseLog,
    range_start: &NaiveDate,
    range_end: &NaiveDate,
) -> bool {
    debug!("checking if {log:?} matches the given date_range: {range_start:?}, {range_end:?}");

    let log_start = log.starts_at.date_naive();
    let log_end = log.ends_at.unwrap_or(Local::now().into()).date_naive();
    log_start >= *range_start && log_end <= *range_end
}

pub fn get_date_info_msg(today: NaiveDate, compare_to: NaiveDate) -> String {
    let diff_days = (today - compare_to).num_days();
    if diff_days == 0 {
        return "Today".to_string();
    }

    if diff_days == 1 {
        return "Yesterday".to_string();
    }

    if diff_days <= 7 {
        return format!("{diff_days} days ago");
    }

    let diff_weeks = diff_days / 7;
    if diff_weeks <= 4 {
        return format!(
            "{diff_weeks} week{} ago",
            if diff_weeks == 1 { "" } else { "s" }
        );
    }

    let diff_months = diff_weeks / 4;
    if diff_months == 1 {
        return "Last month".to_string();
    }

    format!("{diff_months} months ago")
}

#[cfg(test)]
mod tests {
    use super::*;
    use boat_lib::models::log::Log as DatabaseLog;
    use chrono::{NaiveDate, TimeZone, Utc};

    fn make_log(starts: (i32, u32, u32), ends: Option<(i32, u32, u32)>) -> DatabaseLog {
        DatabaseLog {
            id: 1,
            activity_id: 1,
            starts_at: Utc
                .with_ymd_and_hms(starts.0, starts.1, starts.2, 10, 0, 0)
                .unwrap(),
            ends_at: ends.map(|(y, m, d)| Utc.with_ymd_and_hms(y, m, d, 11, 0, 0).unwrap()),
        }
    }

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    // --- resolve_tri_state ---

    #[test]
    fn resolve_tri_state_a_overrides() {
        assert!(resolve_tri_state(true, false, false));
        assert!(resolve_tri_state(true, false, true));
    }

    #[test]
    fn resolve_tri_state_b_overrides() {
        assert!(!resolve_tri_state(false, true, true));
        assert!(!resolve_tri_state(false, true, false));
    }

    #[test]
    fn resolve_tri_state_falls_back_to_c() {
        assert!(resolve_tri_state(false, false, true));
        assert!(!resolve_tri_state(false, false, false));
    }

    #[test]
    fn resolve_tri_state_both_ab_set_uses_c() {
        assert!(resolve_tri_state(true, true, true));
        assert!(!resolve_tri_state(true, true, false));
    }

    // --- tags_str ---

    #[test]
    fn tags_str_empty() {
        assert_eq!(tags_str(&std::collections::HashSet::new()), "");
    }

    #[test]
    fn tags_str_single() {
        let mut tags = std::collections::HashSet::new();
        tags.insert("rust".to_string());
        assert_eq!(tags_str(&tags), "rust");
    }

    #[test]
    fn tags_str_sorted_alphabetically() {
        let mut tags = std::collections::HashSet::new();
        tags.insert("zzz".to_string());
        tags.insert("aaa".to_string());
        tags.insert("mmm".to_string());
        assert_eq!(tags_str(&tags), "aaa,mmm,zzz");
    }

    // --- matches_date ---

    #[test]
    fn matches_date_log_on_same_day() {
        let log = make_log((2024, 4, 15), Some((2024, 4, 15)));
        assert!(matches_date(&log, &date("2024-04-15")));
    }

    #[test]
    fn matches_date_log_on_different_day() {
        let log = make_log((2024, 4, 15), Some((2024, 4, 15)));
        assert!(!matches_date(&log, &date("2024-04-14")));
    }

    #[test]
    fn matches_date_log_spanning_two_days_does_not_match() {
        let log = make_log((2024, 4, 14), Some((2024, 4, 15)));
        assert!(!matches_date(&log, &date("2024-04-14")));
        assert!(!matches_date(&log, &date("2024-04-15")));
    }

    // --- matches_date_range ---

    #[test]
    fn matches_date_range_log_within_bounds() {
        let log = make_log((2024, 4, 12), Some((2024, 4, 12)));
        assert!(matches_date_range(
            &log,
            &date("2024-04-10"),
            &date("2024-04-15")
        ));
    }

    #[test]
    fn matches_date_range_log_at_exact_bounds() {
        let log = make_log((2024, 4, 10), Some((2024, 4, 10)));
        assert!(matches_date_range(
            &log,
            &date("2024-04-10"),
            &date("2024-04-10")
        ));
    }

    #[test]
    fn matches_date_range_start_before_range() {
        let log = make_log((2024, 4, 9), Some((2024, 4, 12)));
        assert!(!matches_date_range(
            &log,
            &date("2024-04-10"),
            &date("2024-04-15")
        ));
    }

    #[test]
    fn matches_date_range_end_after_range() {
        let log = make_log((2024, 4, 12), Some((2024, 4, 16)));
        assert!(!matches_date_range(
            &log,
            &date("2024-04-10"),
            &date("2024-04-15")
        ));
    }

    // --- get_date_info_msg ---

    #[test]
    fn get_date_info_msg_today() {
        let today = date("2024-06-15");
        assert_eq!(get_date_info_msg(today, today), "Today");
    }

    #[test]
    fn get_date_info_msg_yesterday() {
        let today = date("2024-06-15");
        assert_eq!(get_date_info_msg(today, date("2024-06-14")), "Yesterday");
    }

    #[test]
    fn get_date_info_msg_days_ago() {
        let today = date("2024-06-15");
        assert_eq!(get_date_info_msg(today, date("2024-06-12")), "3 days ago");
        // boundary: 7 is still "days ago" (the <= 7 check comes before the weeks check)
        assert_eq!(get_date_info_msg(today, date("2024-06-08")), "7 days ago");
    }

    #[test]
    fn get_date_info_msg_weeks_ago() {
        let today = date("2024-06-15");
        assert_eq!(get_date_info_msg(today, date("2024-06-07")), "1 week ago");
        assert_eq!(get_date_info_msg(today, date("2024-06-01")), "2 weeks ago");
    }

    #[test]
    fn get_date_info_msg_last_month() {
        let today = date("2024-06-15");
        // 35 days ago: diff_weeks = 5, diff_months = 1 → "Last month"
        assert_eq!(get_date_info_msg(today, date("2024-05-11")), "Last month");
    }

    #[test]
    fn get_date_info_msg_months_ago() {
        let today = date("2024-06-15");
        // 75 days ago: diff_weeks = 10, diff_months = 2
        assert_eq!(get_date_info_msg(today, date("2024-04-01")), "2 months ago");
    }
}
