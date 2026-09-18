//! Integration tests for the `list` CLI command.
use anyhow::Result;
use predicates::prelude::*;

use crate::utils::{cli_args_for_temp, run_boat};

mod utils;

#[test]
fn list_with_no_activities_shows_no_data() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["list"], config_path)
        .success()
        .stdout(predicates::str::contains("no available data"));

    Ok(())
}

#[test]
fn list_shows_activity_name() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "ListedTask", "--start-now"], &config_path).success();
    run_boat(["pause"], &config_path).success();

    run_boat(["list"], &config_path)
        .success()
        .stdout(predicates::str::contains("ListedTask"));

    Ok(())
}

#[test]
fn list_with_invalid_date_input_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["list", "--period", "not-a-date"], config_path)
        .failure()
        .stderr(predicates::str::contains("Period presets"));

    Ok(())
}

#[test]
fn list_filter_by_tag_shows_only_matching_activity() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "RustWork", "--tags", "rust"], &config_path).success();
    run_boat(["new", "PythonWork", "--tags", "python"], &config_path).success();
    run_boat(["start", "1"], &config_path).success();
    run_boat(["pause"], &config_path).success();
    run_boat(["start", "2"], &config_path).success();
    run_boat(["pause"], &config_path).success();

    run_boat(["list", "--json", "--filter-by-tags", "rust"], &config_path)
        .success()
        .stdout(predicates::str::contains("RustWork"))
        .stdout(predicates::str::contains("PythonWork").not());

    Ok(())
}

#[test]
fn list_filter_by_nonexistent_tag_shows_no_data() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "SomeWork", "--tags", "rust"], &config_path).success();
    run_boat(["start", "1"], &config_path).success();
    run_boat(["pause"], &config_path).success();

    run_boat(["list", "--filter-by-tags", "nonexistent"], &config_path)
        .success()
        .stdout(predicates::str::contains("no available data"));

    Ok(())
}

#[test]
fn list_filter_by_multiple_tags_requires_all() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    // Activity with both required tags
    run_boat(["new", "FullMatch", "--tags", "rust,backend"], &config_path).success();
    // Activity with only one of the required tags
    run_boat(["new", "PartialMatch", "--tags", "rust"], &config_path).success();
    run_boat(["start", "1"], &config_path).success();
    run_boat(["pause"], &config_path).success();
    run_boat(["start", "2"], &config_path).success();
    run_boat(["pause"], &config_path).success();

    // Filtering by both tags should only return the activity that has all of them
    run_boat(
        ["list", "--json", "--filter-by-tags", "rust,backend"],
        &config_path,
    )
    .success()
    .stdout(predicates::str::contains("FullMatch"))
    .stdout(predicates::str::contains("PartialMatch").not());

    Ok(())
}
