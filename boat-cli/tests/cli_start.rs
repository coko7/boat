//! Integration tests for the `start` CLI command.
use anyhow::Result;
use predicates::prelude::*;

use crate::utils::{cli_args_for_temp, run_boat};

mod utils;

#[test]
fn start_with_nonexistent_id_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["start", "1"], config_path)
        .failure()
        .stderr(predicates::str::contains("does not exist"));

    Ok(())
}

#[test]
fn start_with_no_id_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["start"], config_path)
        .failure()
        .stderr(predicates::str::contains("ACTIVITY_HANDLE"));

    Ok(())
}

#[test]
fn start_when_other_activity_is_running_suceeds() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "Task1"], &config_path).success();
    run_boat(["new", "Task2"], &config_path).success();

    // Start first, then second, which should stop the first
    run_boat(["start", "1"], &config_path).success();
    run_boat(["start", "2"], &config_path)
        .success()
        .stdout(predicates::str::contains("paused").and(predicates::str::contains("started")));

    Ok(())
}

#[test]
fn start_resumes_paused_activity() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "ResumeMe"], &config_path).success();
    run_boat(["start", "1"], &config_path).success();
    run_boat(["pause"], &config_path).success();

    // Resuming the paused activity should succeed and make it current again
    run_boat(["start", "1"], &config_path).success();

    run_boat(["get", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("ResumeMe"));

    Ok(())
}

#[test]
fn start_by_name_succeeds() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "ByName"], &config_path).success();

    run_boat(["start", "ByName"], &config_path).success();

    run_boat(["get", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("ByName"));

    Ok(())
}
