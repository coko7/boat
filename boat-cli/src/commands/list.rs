use anyhow::Result;
use boat_lib::repository::activities_repository as activities;
use chrono::{Datelike, Local};
use log::info;
use rusqlite::Connection;
use std::collections::BTreeMap;

use crate::{
    cli::{self, PeriodInput, args::GroupBy},
    config::Configuration,
    models::{activity_log::PrintableActivityLog, boat_data::BoatData},
    utils::{self, date::DateTimeRenderMode},
};

pub fn list_activity_logs(
    config: &Configuration,
    conn: &Connection,
    args: &cli::FilterActivitiesArgs,
) -> Result<()> {
    let period = args
        .period
        .or(config.commands.list.period)
        .or(config.period)
        .unwrap_or(PeriodInput::Preset(cli::PresetPeriod::AllTime));
    info!("using period: {period}");

    let group_by_value = args
        .group_by
        .or(config.commands.list.group_by)
        .unwrap_or(GroupBy::None);
    info!("grouping by: {group_by_value}");

    let fields = args
        .fields
        .clone()
        .or_else(|| config.commands.list.fields.clone());

    info!("getting all activities");
    let db_acts: Vec<_> = activities::get_all(conn)?;
    let boat_data = BoatData::create_filtered_data(db_acts, period);

    info!("filtering logs by tags");
    let prt_logs = boat_data
        .get_printable_logs()
        .into_iter()
        .filter(|log| {
            if let Some(filter_tags) = &args.filter_by_tags {
                filter_tags
                    .iter()
                    .all(|tag| log.activity.tags.contains(tag))
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    info!("grouping logs based on group_by value");
    let grouped_logs = group_by(&prt_logs, group_by_value);
    if args.use_json_format {
        let json = serde_json::to_string(&grouped_logs)?;
        println!("{json}");
        return Ok(());
    }

    if grouped_logs.is_empty() {
        println!("no available data");
        return Ok(());
    }

    info!("displaying activity logs grouped by date");
    for (group, act_logs) in grouped_logs.iter() {
        let (text, tooltip) = utils::display::get_group_by_display_values(group_by_value, group)?;
        let ribbon = utils::display::format_ascii_ribbon(&text, tooltip.as_deref());
        println!("{ribbon}");
        utils::common::list_printable_items(act_logs, false, fields.as_deref())?;
    }

    Ok(())
}

trait ActivityLog {
    fn starts_at(&self) -> Option<chrono::DateTime<Local>>;
    fn ends_at(&self) -> Option<chrono::DateTime<Local>>;
}

fn group_by(
    activity_logs: &[PrintableActivityLog],
    group_by: GroupBy,
) -> BTreeMap<String, Vec<PrintableActivityLog>> {
    match group_by {
        GroupBy::None => {
            let mut map = BTreeMap::new();
            map.insert("all".to_string(), activity_logs.to_vec());
            map
        }
        GroupBy::Day => group_by_day(activity_logs),
        GroupBy::Week => group_by_week(activity_logs),
        GroupBy::Month => group_by_month(activity_logs),
        GroupBy::Year => group_by_year(activity_logs),
    }
}

fn group_by_day(
    activity_logs: &[PrintableActivityLog],
) -> BTreeMap<String, Vec<PrintableActivityLog>> {
    let mut groups: BTreeMap<_, Vec<_>> = BTreeMap::new();

    for act_log in activity_logs {
        let latest_dt = act_log.log.ends_at.unwrap_or(Local::now());
        let key = DateTimeRenderMode::DateOnly.render_date_time(latest_dt);
        groups.entry(key).or_default().push(act_log.clone());
    }

    groups
}

fn group_by_week(
    activity_logs: &[PrintableActivityLog],
) -> BTreeMap<String, Vec<PrintableActivityLog>> {
    let mut groups: BTreeMap<_, Vec<_>> = BTreeMap::new();

    for act_log in activity_logs {
        let latest_dt = act_log.log.ends_at.unwrap_or(Local::now());
        let iso_week = latest_dt.iso_week();
        let key = format!("{}-W{}", iso_week.year(), iso_week.week());
        groups.entry(key).or_default().push(act_log.clone());
    }

    groups
}

fn group_by_month(
    activity_logs: &[PrintableActivityLog],
) -> BTreeMap<String, Vec<PrintableActivityLog>> {
    let mut groups: BTreeMap<_, Vec<_>> = BTreeMap::new();

    for act_log in activity_logs {
        let latest_dt = act_log.log.ends_at.unwrap_or(Local::now());
        let key = format!("{}-{:02}", latest_dt.year(), latest_dt.month());
        groups.entry(key).or_default().push(act_log.clone());
    }

    groups
}

fn group_by_year(
    activity_logs: &[PrintableActivityLog],
) -> BTreeMap<String, Vec<PrintableActivityLog>> {
    let mut groups: BTreeMap<_, Vec<_>> = BTreeMap::new();

    for act_log in activity_logs {
        let latest_dt = act_log.log.ends_at.unwrap_or(Local::now());
        let key = format!("{}", latest_dt.year());
        groups.entry(key).or_default().push(act_log.clone());
    }

    groups
}
