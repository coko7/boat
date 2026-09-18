use anyhow::{Result, bail};
use boat_lib::repository::activities_repository as activities;
use log::info;
use rusqlite::Connection;
use yansi::Paint;

use crate::{
    cli::{self, PeriodInput},
    config::Configuration,
    models::boat_data::BoatData,
    utils,
};

pub fn show_report(
    config: &Configuration,
    conn: &Connection,
    args: &cli::FilterActivitiesArgs,
) -> Result<()> {
    let period = args
        .period
        .or(config.commands.report.period)
        .or(config.period)
        .unwrap_or(PeriodInput::Preset(cli::PresetPeriod::AllTime));
    info!("using period: {period}");

    if args.group_by.is_some() {
        bail!("grouping is not supported for report command yet");
    }

    // let group_by_value = args
    //     .group_by
    //     .or(config.commands.list.group_by)
    //     .unwrap_or(GroupBy::None);
    // info!("grouping by: {group_by_value}");

    info!("getting all activities");
    let db_acts: Vec<_> = activities::get_all(conn)?;
    let boat_data = BoatData::create_filtered_data(db_acts, period);

    info!("listing activity summaries");
    let prt_acts = boat_data
        .get_printable_activities()
        .into_iter()
        .filter(|act| act.duration > 0)
        .collect::<Vec<_>>();

    // let grouped_acts = utils::common::group_by(&prt_acts, group_by_value);

    info!("showing summary");
    if !args.use_json_format {
        // let date_msg = match args.period {
        //     Some(dt_range) => Some(dt_range.to_string()),
        //     None => args.period.map(|p| p.to_string()),
        //     cli::PeriodInput::Preset(preset_period) => todo!(),
        //     cli::PeriodInput::Single(naive_date) => todo!(),
        //     cli::PeriodInput::Range(range) => range.to_string(),
        // };

        // if let Some(date_msg) = date_msg {
        //     info!("using custom date msg for summary");
        //     println!("{} {}\n", "Summary:".underline(), date_msg.green());
        // }
    }

    let fields = args
        .fields
        .clone()
        .or_else(|| config.commands.report.fields.clone());

    list_activity_summaries(
        &boat_data,
        args.use_json_format,
        &args.filter_by_tags,
        fields.as_deref(),
    )
}

fn list_activity_summaries(
    boat_data: &BoatData,
    use_json: bool,
    filter_by_tags: &Option<Vec<String>>,
    fields: Option<&[String]>,
) -> Result<()> {
    info!("filtering logs by tags");
    let prt_acts = boat_data
        .get_printable_activities()
        .into_iter()
        .filter(|act| act.duration > 0)
        .filter(|act| {
            if let Some(filter_tags) = filter_by_tags {
                filter_tags.iter().all(|tag| act.tags.contains(tag))
            } else {
                true
            }
        })
        .collect();

    info!("listing activity summaries");
    utils::common::list_printable_items(&prt_acts, use_json, fields)?;

    if !use_json && !prt_acts.is_empty() {
        let total_sec: i64 = prt_acts.iter().map(|pa| pa.duration).sum();
        println!(
            "{} {}",
            "Total:".underline(),
            utils::date::pretty_format_duration(total_sec, false).green()
        );
    }

    Ok(())
}
