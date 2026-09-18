//! Integration tests for the `report` CLI command.
use anyhow::Result;
use predicates::prelude::*;

use crate::utils::{cli_args_for_temp, run_boat};

mod utils;

#[test]
fn report_with_no_activities_shows_no_data() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["report"], config_path)
        .success()
        .stdout(predicates::str::contains("no available data"));

    Ok(())
}

#[test]
fn report_filter_by_nonexistent_tag_shows_no_data() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(
        ["new", "RustWork", "--tags", "rust", "--start-now"],
        &config_path,
    )
    .success();
    run_boat(["pause"], &config_path).success();

    // "python" tag doesn't exist on any activity, so report should be empty
    run_boat(["report", "--filter-by-tags", "python"], &config_path)
        .success()
        .stdout(predicates::str::contains("no available data"));

    Ok(())
}

#[test]
fn report_filter_by_tag_excludes_untagged_activities() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // Activity with matching tag
    run_boat(
        ["new", "Tagged", "--tags", "keep", "--start-now"],
        &config_path,
    )
    .success();
    run_boat(["pause"], &config_path).success();

    // Activity without any tag
    run_boat(["new", "Untagged", "--start-now"], &config_path).success();
    run_boat(["pause"], &config_path).success();

    // JSON output should not contain the untagged activity name when filtering
    run_boat(
        ["report", "--json", "--filter-by-tags", "keep"],
        &config_path,
    )
    .success()
    .stdout(predicates::str::contains("Untagged").not());

    Ok(())
}
