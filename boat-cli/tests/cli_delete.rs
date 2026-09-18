//! Integration tests for the `delete` CLI command.
use anyhow::Result;
use predicates::prelude::*;

use crate::utils::{cli_args_for_temp, run_boat};

mod utils;

#[test]
fn delete_existing_succeeds() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // Create, start and pause to make listable
    run_boat(["new", "to-delete"], &config_path).success();
    run_boat(["start", "1"], &config_path).success();
    run_boat(["pause"], &config_path).success();
    run_boat(["list"], &config_path)
        .success()
        .stdout(predicates::str::contains("to-delete"));

    // Delete
    run_boat(["delete", "1", "--no-confirm"], &config_path).success();

    // Should not appear in the list anymore
    run_boat(["list", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("to-delete").not());

    Ok(())
}

#[test]
fn delete_nonexistent_id_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["delete", "1"], &config_path)
        .failure()
        .stderr(predicates::str::contains("does not exist"));

    Ok(())
}

#[test]
fn delete_with_missing_id_arg_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;
    run_boat(["delete"], &config_path)
        .failure()
        .stderr(predicates::str::contains("ID"));

    Ok(())
}

#[test]
fn delete_running_activity_succeeds() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "ActiveTask", "--start-now"], &config_path).success();

    run_boat(["delete", "1", "--no-confirm"], &config_path).success();

    run_boat(["list", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("ActiveTask").not());

    Ok(())
}
