//! Integration tests for the `cancel` CLI command.
use anyhow::Result;

use crate::utils::{cli_args_for_temp, run_boat};

mod utils;

#[test]
fn cancel_running_activity_succeeds() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // Create & start activity
    run_boat(["new", "to-cancel", "--start-now"], &config_path).success();

    // Cancel
    run_boat(["cancel", "--no-confirm"], &config_path)
        .success()
        .stdout(predicates::str::contains("cancelled"));

    // Should be no running activity
    run_boat(["get", "--json"], &config_path)
        .failure()
        .stderr(predicates::str::contains("no current activity"));

    Ok(())
}

#[test]
fn cancel_when_none_running_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // No activity is running
    run_boat(["cancel"], &config_path)
        .failure()
        .stderr(predicates::str::contains("no current activity"));

    Ok(())
}
