//! Integration tests for the `modify` CLI command.
use anyhow::Result;
use predicates::prelude::*;

use crate::utils::{cli_args_for_temp, run_boat};

mod utils;

#[test]
fn modify_name_succeeds() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // Create activity and start
    run_boat(["new", "OldName", "--start-now"], &config_path).success();

    // Modify name
    run_boat(
        ["modify", "1", "--name", "NewName", "--no-confirm"],
        &config_path,
    )
    .success();

    // Confirm change in `list`
    run_boat(["list", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("NewName"));

    Ok(())
}

#[test]
fn modify_description_succeeds() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // Create activity
    run_boat(["new", "TestAct", "--start-now"], &config_path).success();

    // Modify description
    run_boat(
        [
            "modify",
            "1",
            "--description",
            "an activity for tests",
            "--no-confirm",
        ],
        &config_path,
    );

    // Confirm change in `list`
    run_boat(["list", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("an activity for tests"));

    Ok(())
}

#[test]
fn modify_tags_succeeds() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // Create activity
    run_boat(["new", "Taggy", "--start-now"], &config_path).success();

    // Modify tags
    run_boat(
        [
            "modify",
            "1",
            "--tags",
            "testing",
            "write-tests",
            "foo",
            "--no-confirm",
        ],
        &config_path,
    );

    // Confirm change in `list`
    run_boat(["list", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("testing").and(predicates::str::contains("write-tests")));

    Ok(())
}

#[test]
fn modify_nonexistent_id_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // No activities at all
    run_boat(
        ["modify", "1", "--name", "Nobody", "--no-confirm"],
        &config_path,
    )
    .failure()
    .stderr(predicates::str::contains("does not exist"));

    Ok(())
}

#[test]
fn modify_no_field_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // Create activity
    run_boat(["new", "Hello"], &config_path).success();

    // Attempt modify with no field to change
    run_boat(["modify", "1", "--no-confirm"], &config_path)
        .failure()
        .stderr(predicates::str::contains(
            "the following required arguments were not provided",
        ));

    Ok(())
}

#[test]
fn modify_missing_id_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // Try to modify without specifying an ID
    run_boat(["modify", "--name", "Nope", "--no-confirm"], &config_path)
        .failure()
        .stderr(predicates::str::contains("ID"));

    Ok(())
}
