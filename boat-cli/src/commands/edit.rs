use anyhow::{Context, Result, bail, ensure};
use boat_lib::{
    models::log::Log as DatabaseLog,
    repository::logs_repository as logs,
    repository::{Id, activities_repository as activities},
};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Timelike, Utc};
use dialoguer::Confirm;
use log::{debug, info};
use rusqlite::Connection;
use std::{env, fs, path::PathBuf, process::Command};
use yansi::Paint;

use crate::{
    cli::{self, EditLogsArgs, PeriodInput},
    config::Configuration,
    models::boat_data::BoatData,
    utils,
};

pub fn edit(
    config: &Configuration,
    conn: &mut rusqlite::Connection,
    args: &EditLogsArgs,
) -> Result<()> {
    let period = args
        .period
        .or(config.commands.edit.period)
        .or(config.period)
        .unwrap_or(PeriodInput::Preset(cli::PresetPeriod::AllTime));
    info!("using period: {period}");

    let include_instructions = utils::common::resolve_tri_state(
        args.show_instructions,
        args.hide_instructions,
        config.commands.edit.show_instructions,
    );
    info!("include instructions? {include_instructions}");

    let include_activity_definitions = utils::common::resolve_tri_state(
        args.show_activity_definitions,
        args.hide_activity_definitions,
        config.commands.edit.show_activity_definitions,
    );
    info!("include activity definitions? {include_activity_definitions}");

    let ask_for_confirmation = utils::common::resolve_tri_state(
        args.confirm,
        args.no_confirm,
        config.commands.edit.confirm,
    );
    info!("ask user for confirmation? {ask_for_confirmation}");

    let all_acts = activities::get_all(conn)?;
    let boat_data = BoatData::create_filtered_data(all_acts, period);
    let default_content = boat_data.to_csv_str(include_instructions, include_activity_definitions);
    let edit_file_path = create_tmp_edit_file(&default_content)?;

    let editor = args
        .editor
        .clone()
        .or_else(|| env::var("EDITOR").ok())
        .context("No editor specified and EDITOR is not set")?;

    resolve_editor(&editor)?;
    info!("launching editor: {editor}");
    let status = Command::new(editor).arg(&edit_file_path).status()?;

    ensure!(
        status.success(),
        "edit-editor exited with non-zero status: {status}"
    );

    ensure!(
        fs::exists(&edit_file_path)?,
        format!(
            "temporary edit file was removed unexpectedly: {}",
            edit_file_path.display()
        )
    );

    let modified_content = fs::read_to_string(&edit_file_path)?;
    debug!("modified content:\n{}", modified_content);

    fs::remove_file(&edit_file_path)?;
    debug!("temporary edit file removed: {}", edit_file_path.display());

    let updated_logs = convert_modified_content_to_log_lines(&modified_content)?;
    debug!(
        "converted modified content to log lines: {:#?}",
        updated_logs
    );

    let original_logs = boat_data
        .logs
        .values()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    debug!("original logs for validation: {:#?}", original_logs);

    let to_update = try_generate_edit_diffs(&updated_logs, &original_logs)?;
    debug!("generated edit diffs: {:#?}", to_update);

    if to_update.is_empty() {
        println!("no changes detected, nothing to update");
        return Ok(());
    }

    if ask_for_confirmation && !confirm_changes(&to_update)? {
        println!("user aborted the operation, no changes will be made");
        return Ok(());
    }

    let updated = batch_update_activity_logs(conn, &to_update)?;
    println!("successfully updated {} log entries", updated);

    Ok(())
}

fn confirm_changes(to_update: &[EditLogDiff]) -> Result<bool> {
    pretty_print_edit_diffs(to_update);

    let to_update_count = to_update.len();
    let prompt = format!(
        "You are about to update {} log entries. Do you want to proceed?",
        to_update_count
    );

    let proceed = Confirm::new()
        .with_prompt(Paint::yellow(&prompt).to_string())
        .default(false)
        .interact()?;

    Ok(proceed)
}

#[derive(Debug)]
struct EditLogDiff {
    log_id: Id,
    starts_at_current: DateTime<Utc>,
    starts_at_new: Option<DateTime<Utc>>,
    ends_at_current: Option<DateTime<Utc>>,
    ends_at_new: Option<Option<DateTime<Utc>>>,
}

fn resolve_editor(editor: &str) -> Result<()> {
    if editor.is_empty() {
        bail!(
            "Default text editor not found. Please refer to your shell documentation to set your EDITOR environment variable."
        );
    }

    which::which(editor).with_context(|| format!("EDITOR command not found: {editor}"))?;
    Ok(())
}

fn pretty_print_edit_diffs(diffs: &[EditLogDiff]) {
    println!("Detected changes:");
    for diff in diffs {
        let starts_at_change = if let Some(starts_at_new) = diff.starts_at_new {
            format!("starts_at: {} -> {}", diff.starts_at_current, starts_at_new)
        } else {
            format!("starts_at: {} (no change)", diff.starts_at_current)
        };

        let ends_at_change = if let Some(ends_at_new_opt) = diff.ends_at_new {
            if let Some(ends_at_new) = ends_at_new_opt {
                format!("ends_at: {:?} -> {:?}", diff.ends_at_current, ends_at_new)
            } else {
                format!(
                    "ends_at: {:?} -> None (log is now open-ended)",
                    diff.ends_at_current
                )
            }
        } else {
            format!("ends_at: {:?} (no change)", diff.ends_at_current)
        };

        println!(
            "Log ID {}: {}, {}",
            diff.log_id, starts_at_change, ends_at_change
        );
    }
}

fn date_time_opt_loose_eq<Tz>(
    dt1: &Option<DateTime<Tz>>,
    dt2: &Option<DateTime<Tz>>,
) -> Result<bool>
where
    Tz: chrono::TimeZone,
    Tz::Offset: std::fmt::Display,
{
    let dt1_rounded = dt1
        .as_ref()
        .map(|dt| {
            dt.with_second(0)
                .with_context(|| format!("failed to reset seconds on dt: {dt}"))
        })
        .transpose()?;

    let dt2_rounded = dt2
        .as_ref()
        .map(|dt| {
            dt.with_second(0)
                .with_context(|| format!("failed to reset seconds on dt: {dt}"))
        })
        .transpose()?;

    Ok(dt1_rounded == dt2_rounded)
}

fn try_generate_edit_diffs(
    edited_db_logs: &[DatabaseLog],
    original_logs: &[DatabaseLog],
) -> Result<Vec<EditLogDiff>> {
    info!("generating diffs between edited logs and original logs for validation");

    let mut diffs = Vec::new();

    ensure!(
        edited_db_logs.len() == original_logs.len(),
        format!(
            "number of log entries has changed: original had {}, but edited has {}",
            original_logs.len(),
            edited_db_logs.len()
        ),
    );

    if edited_db_logs
        .iter()
        .filter(|log| log.ends_at.is_none())
        .count()
        > 1
    {
        bail!("more than one log entry is open-ended (missing ends_at), which is not allowed");
    }

    for log in edited_db_logs {
        let matching_original_log = original_logs
            .iter()
            .find(|orig_log| orig_log.id == log.id)
            .with_context(|| {
                format!("Log with ID {} does not exist in the original data", log.id)
            })?;

        ensure!(
            matching_original_log.activity_id == log.activity_id,
            format!(
                "Log with ID {} has a different activity_id ({}) than the original ({})",
                log.id, log.activity_id, matching_original_log.activity_id
            )
        );

        ensure!(
            log.starts_at <= log.ends_at.unwrap_or_else(chrono::Utc::now),
            format!(
                "Log with ID {} has starts_at ({}) after ends_at ({})",
                log.id,
                log.starts_at,
                log.ends_at.unwrap_or_default()
            )
        );

        let mut updated_starts_at = None;
        if !date_time_opt_loose_eq(&Some(log.starts_at), &Some(matching_original_log.starts_at))? {
            info!(
                "updating log ID {}: starts_at changed from {} to {}",
                log.id, matching_original_log.starts_at, log.starts_at
            );
            updated_starts_at = Some(log.starts_at);
        }

        let mut updated_ends_at = None;

        if !date_time_opt_loose_eq(&log.ends_at, &matching_original_log.ends_at)? {
            info!(
                "updating log ID {}: ends_at changed from {:?} to {:?}",
                log.id, matching_original_log.ends_at, log.ends_at
            );
            updated_ends_at = Some(log.ends_at);
        }

        if updated_starts_at.is_some() || updated_ends_at.is_some() {
            diffs.push(EditLogDiff {
                log_id: log.id,
                starts_at_current: matching_original_log.starts_at,
                starts_at_new: updated_starts_at,
                ends_at_current: matching_original_log.ends_at,
                ends_at_new: updated_ends_at,
            });
        }
    }

    Ok(diffs)
}

fn batch_update_activity_logs(conn: &mut Connection, diffs: &[EditLogDiff]) -> Result<usize> {
    info!("validating user-edited data");

    let mut updated = 0;
    for diff in diffs {
        if let Some(starts_at_new) = diff.starts_at_new {
            info!(
                "updating log ID {}: starts_at from {} to {}",
                diff.log_id, diff.starts_at_current, starts_at_new
            );
            logs::update_start(conn, diff.log_id, starts_at_new)?;
            updated += 1;
        }

        if let Some(ends_at_new) = diff.ends_at_new {
            info!(
                "updating log ID {}: ends_at from {:?} to {:?}",
                diff.log_id, diff.ends_at_current, ends_at_new
            );
            logs::update_end(conn, diff.log_id, ends_at_new)?;
            updated += 1;
        }
    }

    Ok(updated)
}

fn convert_modified_content_to_log_lines(content: &str) -> Result<Vec<DatabaseLog>> {
    info!("converting modified content to log lines");
    let lines_without_comments = content
        .lines()
        .filter(|line| !line.is_empty() && !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>();

    let mut logs = Vec::new();
    for log_line in lines_without_comments {
        debug!("processing line: {}", log_line);
        let parts: Vec<&str> = log_line.split(',').map(|s| s.trim()).collect();

        ensure!(
            parts.len() == 4,
            format!(
                "expected 4 columns in the CSV, but got {}: {}",
                parts.len(),
                log_line
            )
        );

        let activity_id = parts[0].parse::<Id>().with_context(|| {
            format!(
                "failed to parse activity ID from '{}': {}",
                parts[0], log_line
            )
        })?;

        let log_id = parts[1]
            .parse::<Id>()
            .with_context(|| format!("failed to parse log ID from '{}': {}", parts[1], log_line))?;

        let log_starts_at_naive = NaiveDateTime::parse_from_str(parts[2], "%Y-%m-%d %H:%M")
            .with_context(|| {
                format!(
                    "failed to parse starts_at from '{}': {}",
                    parts[2], log_line
                )
            })?;
        let log_starts_at = Local
            .from_local_datetime(&log_starts_at_naive)
            .single()
            .with_context(|| {
                format!(
                    "failed to convert starts_at to local datetime from '{}': {}",
                    parts[2], log_line
                )
            })?;

        let log_ends_at_naive_opt = if parts[3].is_empty() {
            None
        } else {
            Some(
                NaiveDateTime::parse_from_str(parts[3], "%Y-%m-%d %H:%M").with_context(|| {
                    format!("failed to parse ends_at from '{}': {}", parts[3], log_line)
                })?,
            )
        };
        let log_ends_at = if let Some(naive_dt) = log_ends_at_naive_opt {
            Some(
                Local
                    .from_local_datetime(&naive_dt)
                    .single()
                    .with_context(|| {
                        format!(
                            "failed to convert ends_at to local datetime from '{}': {}",
                            parts[3], log_line
                        )
                    })?,
            )
        } else {
            None
        };

        logs.push(DatabaseLog {
            id: log_id,
            activity_id,
            starts_at: log_starts_at.with_timezone(&chrono::Utc),
            ends_at: log_ends_at.map(|dt| dt.with_timezone(&chrono::Utc)),
        });
    }

    Ok(logs)
}

fn create_tmp_edit_file(content: &str) -> Result<PathBuf> {
    let tmp_dir = env::temp_dir();
    let file_path = tmp_dir.join("boat_edit_logs_tmp.csv");
    fs::write(&file_path, content)?;
    info!(
        "created temporary file for editing: {}",
        file_path.display()
    );
    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use boat_lib::models::log::Log as DatabaseLog;
    use chrono::{Local, NaiveDateTime, TimeZone, Utc};

    fn make_log(id: Id, activity_id: Id, starts: &str, ends: Option<&str>) -> DatabaseLog {
        let starts_at = Local
            .from_local_datetime(&NaiveDateTime::parse_from_str(starts, "%Y-%m-%d %H:%M").unwrap())
            .single()
            .unwrap()
            .with_timezone(&Utc);
        let ends_at = ends.map(|e| {
            Local
                .from_local_datetime(&NaiveDateTime::parse_from_str(e, "%Y-%m-%d %H:%M").unwrap())
                .single()
                .unwrap()
                .with_timezone(&Utc)
        });
        DatabaseLog {
            id,
            activity_id,
            starts_at,
            ends_at,
        }
    }

    #[test]
    fn test_convert_modified_content_to_log_lines() {
        let csv = "\
1, 10, 2024-06-01 10:00, 2024-06-01 11:00
2, 20, 2024-06-02 12:00,
";
        let logs = convert_modified_content_to_log_lines(csv).unwrap();

        // test first log
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].id, 10);
        assert_eq!(logs[0].activity_id, 1);
        assert!(logs[0].ends_at.is_some());

        // test second log
        assert_eq!(logs[1].id, 20);
        assert!(logs[1].ends_at.is_none());
    }

    #[test]
    fn test_try_generate_edit_diffs_detects_changes() {
        let orig = vec![make_log(
            1,
            100,
            "2024-06-01 10:00",
            Some("2024-06-01 11:00"),
        )];
        let edited = vec![make_log(
            1,
            100,
            "2024-06-01 10:05",
            Some("2024-06-01 11:00"),
        )];

        let diffs = try_generate_edit_diffs(&edited, &orig).unwrap();
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].log_id, 1);
        assert_eq!(diffs[0].starts_at_new, Some(edited[0].starts_at));
        assert!(diffs[0].ends_at_new.is_none());
    }

    #[test]
    fn test_date_time_opt_loose_eq_ignores_seconds() {
        let dt1 = Some(Utc.with_ymd_and_hms(2024, 6, 1, 10, 0, 30).unwrap());
        let dt2 = Some(Utc.with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap());
        assert!(date_time_opt_loose_eq(&dt1, &dt2).unwrap());
    }

    #[test]
    fn test_convert_modified_content_to_log_lines_invalid() {
        let csv = "1, 10, not-a-date, 2024-06-01 11:00";
        assert!(convert_modified_content_to_log_lines(csv).is_err());
    }

    // --- resolve_editor ---

    #[test]
    fn resolve_editor_empty_string_fails() {
        assert!(resolve_editor("").is_err());
    }

    #[test]
    fn resolve_editor_nonexistent_command_fails() {
        assert!(resolve_editor("definitely-not-a-real-editor-xyz").is_err());
    }

    // --- try_generate_edit_diffs error paths ---

    #[test]
    fn try_generate_edit_diffs_length_mismatch_fails() {
        let orig = vec![make_log(1, 1, "2024-06-01 10:00", Some("2024-06-01 11:00"))];
        let edited = vec![
            make_log(1, 1, "2024-06-01 10:00", Some("2024-06-01 11:00")),
            make_log(2, 1, "2024-06-01 12:00", Some("2024-06-01 13:00")),
        ];
        assert!(try_generate_edit_diffs(&edited, &orig).is_err());
    }

    #[test]
    fn try_generate_edit_diffs_multiple_open_ended_logs_fails() {
        let orig = vec![
            make_log(1, 1, "2024-06-01 10:00", None),
            make_log(2, 1, "2024-06-01 12:00", None),
        ];
        assert!(try_generate_edit_diffs(&orig, &orig).is_err());
    }

    #[test]
    fn try_generate_edit_diffs_unknown_log_id_fails() {
        let orig = vec![make_log(1, 1, "2024-06-01 10:00", Some("2024-06-01 11:00"))];
        let edited = vec![make_log(
            99,
            1,
            "2024-06-01 10:00",
            Some("2024-06-01 11:00"),
        )];
        assert!(try_generate_edit_diffs(&edited, &orig).is_err());
    }

    #[test]
    fn try_generate_edit_diffs_activity_id_changed_fails() {
        let orig = vec![make_log(1, 1, "2024-06-01 10:00", Some("2024-06-01 11:00"))];
        let edited = vec![make_log(
            1,
            999,
            "2024-06-01 10:00",
            Some("2024-06-01 11:00"),
        )];
        assert!(try_generate_edit_diffs(&edited, &orig).is_err());
    }

    #[test]
    fn try_generate_edit_diffs_starts_at_after_ends_at_fails() {
        let orig = vec![make_log(1, 1, "2024-06-01 10:00", Some("2024-06-01 11:00"))];
        let edited = vec![make_log(1, 1, "2024-06-01 12:00", Some("2024-06-01 11:00"))];
        assert!(try_generate_edit_diffs(&edited, &orig).is_err());
    }

    // --- try_generate_edit_diffs happy paths ---

    #[test]
    fn try_generate_edit_diffs_no_change_returns_empty() {
        let logs = vec![make_log(1, 1, "2024-06-01 10:00", Some("2024-06-01 11:00"))];
        let diffs = try_generate_edit_diffs(&logs, &logs).unwrap();
        assert!(diffs.is_empty());
    }

    #[test]
    fn try_generate_edit_diffs_detects_ends_at_change() {
        let orig = vec![make_log(1, 1, "2024-06-01 10:00", Some("2024-06-01 11:00"))];
        let edited = vec![make_log(1, 1, "2024-06-01 10:00", Some("2024-06-01 12:00"))];
        let diffs = try_generate_edit_diffs(&edited, &orig).unwrap();
        assert_eq!(diffs.len(), 1);
        assert!(diffs[0].ends_at_new.is_some());
        assert!(diffs[0].starts_at_new.is_none());
    }

    #[test]
    fn try_generate_edit_diffs_detects_log_closed_to_open() {
        let orig = vec![make_log(1, 1, "2024-06-01 10:00", Some("2024-06-01 11:00"))];
        let edited = vec![make_log(1, 1, "2024-06-01 10:00", None)];
        let diffs = try_generate_edit_diffs(&edited, &orig).unwrap();
        assert_eq!(diffs.len(), 1);
        // ends_at_new is Some(None) — the log was re-opened
        assert_eq!(diffs[0].ends_at_new, Some(None));
    }

    // --- date_time_opt_loose_eq ---

    #[test]
    fn date_time_opt_loose_eq_both_none_are_equal() {
        let none: Option<DateTime<Utc>> = None;
        assert!(date_time_opt_loose_eq(&none, &none).unwrap());
    }

    #[test]
    fn date_time_opt_loose_eq_some_and_none_are_not_equal() {
        let some = Some(Utc.with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap());
        let none = None;
        assert!(!date_time_opt_loose_eq(&some, &none).unwrap());
        assert!(!date_time_opt_loose_eq(&none, &some).unwrap());
    }

    #[test]
    fn date_time_opt_loose_eq_different_minutes_are_not_equal() {
        let dt1 = Some(Utc.with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap());
        let dt2 = Some(Utc.with_ymd_and_hms(2024, 6, 1, 10, 1, 0).unwrap());
        assert!(!date_time_opt_loose_eq(&dt1, &dt2).unwrap());
    }
}
