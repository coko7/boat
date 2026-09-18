//! Integration tests for the `pause` CLI command.
use anyhow::Result;

use crate::utils::{cli_args_for_temp, run_boat};

mod utils;

#[test]
fn pause_when_nothing_running_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["pause"], config_path)
        .success()
        .stdout(predicates::str::contains("no current activity"));

    Ok(())
}

#[test]
fn pause_running_activity_succeeds() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "PauseMe", "--start-now"], &config_path).success();

    run_boat(["pause"], &config_path)
        .success()
        .stdout(predicates::str::contains("paused"));

    run_boat(["get", "--json"], &config_path)
        .failure()
        .stderr(predicates::str::contains("no current activity"));

    Ok(())
}

#[test]
fn pause_outputs_activity_name() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "NamedActivity", "--start-now"], &config_path).success();

    run_boat(["pause"], &config_path)
        .success()
        .stdout(predicates::str::contains("NamedActivity"));

    Ok(())
}
