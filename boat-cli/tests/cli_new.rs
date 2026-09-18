//! Integration tests for the `new` CLI command.
use anyhow::Result;
use predicates::prelude::*;

use crate::utils::{cli_args_for_temp, run_boat};

mod utils;

#[test]
fn new_without_name_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new"], config_path).failure().stderr(
        predicates::str::contains("error").or(predicates::str::contains("required arguments")),
    );

    Ok(())
}

#[test]
fn new_with_name_succeeds() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "MyTask"], &config_path).success();

    // Start and pause so it has a log and appears in list output
    run_boat(["start", "1"], &config_path).success();
    run_boat(["pause"], &config_path).success();

    run_boat(["list", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("MyTask"));

    Ok(())
}

#[test]
fn new_with_start_now_starts_activity() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "AutoStarted", "--start-now"], &config_path).success();

    run_boat(["get", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("AutoStarted"));

    Ok(())
}

#[test]
fn new_with_no_auto_start_does_not_start() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "NotStarted", "--no-start-now"], &config_path).success();

    run_boat(["get", "--json"], &config_path)
        .failure()
        .stderr(predicates::str::contains("no current activity"));

    Ok(())
}

#[test]
fn new_with_tags_creates_tagged_activity() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "TaggedTask", "--tags", "rust,cli"], &config_path).success();
    run_boat(["start", "1"], &config_path).success();
    run_boat(["pause"], &config_path).success();

    run_boat(["list", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("rust").and(predicates::str::contains("cli")));

    Ok(())
}

#[test]
fn new_with_description_creates_described_activity() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(
        [
            "new",
            "DescribedTask",
            "--description",
            "a very useful task",
        ],
        &config_path,
    )
    .success();
    run_boat(["start", "1"], &config_path).success();
    run_boat(["pause"], &config_path).success();

    run_boat(["list", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("a very useful task"));

    Ok(())
}
