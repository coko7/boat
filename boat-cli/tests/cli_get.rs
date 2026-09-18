//! Integration tests for the `get` CLI command.
use anyhow::Result;

use crate::utils::{cli_args_for_temp, run_boat};

mod utils;

#[test]
fn get_running_activity_plain_succeeds() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // Create, start activity and get
    run_boat(["new", "get-act-plain", "--start-now"], &config_path).success();
    run_boat(["start", "1"], &config_path).success();
    run_boat(["get"], &config_path).stdout(predicates::str::contains("get-act-plain"));

    Ok(())
}

#[test]
fn get_running_activity_json_succeeds() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // Create, start activity and get as JSON
    run_boat(["new", "get-act-json", "--start-now"], &config_path).success();
    run_boat(["get", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("get-act-json"));

    Ok(())
}

#[test]
fn get_when_no_current_activity_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // No activities have been started
    run_boat(["get", "--json"], &config_path)
        .failure()
        .stderr(predicates::str::contains("no current activity"));

    Ok(())
}

#[test]
fn get_after_pause_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "WasRunning", "--start-now"], &config_path).success();
    run_boat(["pause"], &config_path).success();

    run_boat(["get", "--json"], &config_path)
        .failure()
        .stderr(predicates::str::contains("no current activity"));

    Ok(())
}
