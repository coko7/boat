use anyhow::{Context, Result};
use boat_lib::{models::activity::Activity as DatabaseActivity, repository::Id};
use chrono::{DateTime, Local, NaiveDate};
use yansi::Paint;

use crate::{
    cli::args::GroupBy,
    utils::{self, date::DateTimeRenderMode},
};

pub fn format_ascii_ribbon(text: &str, tooltip_text: Option<&str>) -> String {
    let top_bot = format!("    *{}*\n", "-".repeat(text.len() + 2));

    let arrow_part = match tooltip_text {
        Some(tt) => format!("|=========( {tt} )====================>"),
        None => "|==================================>".to_string(),
    };

    format!(
        "{}{} {} {}\n{}",
        top_bot.blue(),
        "====|".blue(),
        text.cyan().bold(),
        arrow_part.blue(),
        top_bot.blue(),
    )
}

pub fn get_group_by_display_values(
    group_by: GroupBy,
    key: &str,
) -> Result<(String, Option<String>)> {
    let tuple = match group_by {
        GroupBy::None => ("ALL".to_string(), None),
        GroupBy::Day => {
            let diff_msg = utils::common::get_date_info_msg(
                Local::now().date_naive(),
                NaiveDate::parse_from_str(key, "%Y-%m-%d")?,
            );

            // format!("Day {}", DateTimeRenderMode::DateOnly.render_date_time_str(key)), Some(diff_msg))
            (key.to_string(), Some(diff_msg))
        }
        GroupBy::Week => {
            let week_num = key.split("-W").nth(1).unwrap_or(key);
            let first_day_of_week = NaiveDate::parse_from_str(
                &format!("{}-W{}-1", key.split("-W").next().unwrap_or(key), week_num),
                "%Y-W%W-%u",
            )?
            .format("%b %d, %Y")
            .to_string();
            let last_day_of_week = NaiveDate::parse_from_str(
                &format!("{}-W{}-7", key.split("-W").next().unwrap_or(key), week_num),
                "%Y-W%W-%u",
            );

            (
                format!("Week {week_num}"),
                Some(format!(
                    "{} - {}",
                    first_day_of_week,
                    last_day_of_week.unwrap().format("%b %d, %Y")
                )),
            )
        }
        GroupBy::Month => {
            let first_day_of_month = format!("{}-01", key);
            (
                NaiveDate::parse_from_str(&first_day_of_month, "%Y-%m-%d")?
                    .format("%B %Y")
                    .to_string(),
                None,
            )
        }
        GroupBy::Year => (key.to_string(), None),
    };
    Ok(tuple)
}

pub fn created_activity_msg(activity: &DatabaseActivity) -> String {
    format!(
        "{} new #{} \"{}\"",
        "created".cyan(),
        activity.id,
        activity.name
    )
}

pub fn paused_activity_msg(
    activity: &DatabaseActivity,
    stop_dt: DateTime<Local>,
) -> Result<String> {
    let stop_dt = DateTimeRenderMode::TimeOnly.render_date_time(stop_dt);
    Ok(format!(
        "{} #{} \"{}\" at {stop_dt}",
        "paused".bright_blue(),
        activity.id,
        activity.name
    ))
}

pub fn started_activity_msg(activity: &DatabaseActivity, start_dt: DateTime<Local>) -> String {
    let start_dt = DateTimeRenderMode::TimeOnly.render_date_time(start_dt);
    format!(
        "{} #{} \"{}\" at {start_dt}",
        "started".green(),
        activity.id,
        activity.name
    )
}

pub fn invalid_activity_name(activity_name: &str) -> String {
    format!(
        "the activity name cannot only contain digits: \"{}\"",
        activity_name
    )
    .red()
    .to_string()
}

pub fn activity_id_does_not_exist(id: Id) -> String {
    format!("#{id} does not exist").red().to_string()
}

pub fn invalid_activity_id_format(id_str: &str) -> String {
    format!("invalid activity ID format: \"{}\"", id_str)
        .red()
        .to_string()
}

pub fn deleted_activity_msg(activity: &DatabaseActivity) -> String {
    format!("{} #{} \"{}\"", "deleted".red(), activity.id, activity.name)
}

pub fn modified_activity_msg(activity: &DatabaseActivity) -> String {
    format!(
        "{} #{} \"{}\"",
        "modified".yellow(),
        activity.id,
        activity.name
    )
}

pub fn cancelled_activity_msg(activity: &DatabaseActivity) -> String {
    format!(
        "{} #{} \"{}\"",
        "cancelled".bright_white(),
        activity.id,
        activity.name
    )
}

pub fn current_activity_msg(activity: &DatabaseActivity) -> Result<String> {
    let desc = if let Some(desc) = &activity.description {
        format!(" ({desc})").to_string()
    } else {
        "".to_string()
    };

    let started_at = activity
        .logs
        .iter()
        .find(|l| l.ends_at.is_none())
        .context("there should be an ongoing log")?
        .starts_at
        .with_timezone(&Local);

    let duration_secs = (Local::now() - started_at).num_seconds();

    let tags_str = if !activity.tags.is_empty() {
        let tags_str = utils::common::tags_str(&activity.tags);
        format!(" [{tags_str}]")
    } else {
        "".to_string()
    };

    Ok(format!(
        "{} #{} \"{}\"{}{}: {} -> {} ({})",
        "current:".blue(),
        activity.id,
        activity.name,
        desc,
        tags_str,
        DateTimeRenderMode::TimeOnly.render_date_time(started_at),
        "Now",
        utils::date::pretty_format_duration(duration_secs, true),
    ))
}

pub fn no_current_act_msg() -> String {
    "no current activity".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_group_by_display_values_none_returns_all() {
        let (label, tooltip) = get_group_by_display_values(GroupBy::None, "anything").unwrap();
        assert_eq!(label, "ALL");
        assert!(tooltip.is_none());
    }

    #[test]
    fn get_group_by_display_values_year_returns_key() {
        let (label, tooltip) = get_group_by_display_values(GroupBy::Year, "2024").unwrap();
        assert_eq!(label, "2024");
        assert!(tooltip.is_none());
    }

    #[test]
    fn get_group_by_display_values_month_formats_name() {
        let (label, tooltip) = get_group_by_display_values(GroupBy::Month, "2024-04").unwrap();
        assert_eq!(label, "April 2024");
        assert!(tooltip.is_none());
    }

    #[test]
    fn get_group_by_display_values_month_invalid_key_fails() {
        assert!(get_group_by_display_values(GroupBy::Month, "not-a-month").is_err());
    }

    #[test]
    fn get_group_by_display_values_week_label_and_date_range() {
        let (label, tooltip) = get_group_by_display_values(GroupBy::Week, "2024-W10").unwrap();
        assert_eq!(label, "Week 10");
        // 2024 starts on a Monday, so week 10 runs Mar 04–Mar 10
        let tooltip = tooltip.unwrap();
        assert!(tooltip.contains("Mar 04, 2024"), "got: {tooltip}");
        assert!(tooltip.contains("Mar 10, 2024"), "got: {tooltip}");
    }

    #[test]
    fn get_group_by_display_values_week_invalid_key_fails() {
        assert!(get_group_by_display_values(GroupBy::Week, "not-a-week").is_err());
    }
}
