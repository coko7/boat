//! Basic smoke tests: help, version, invalid command
use anyhow::Result;
use predicates::prelude::*;

use crate::utils::{cli_args_for_temp, run_boat};

mod utils;

#[test]
fn test_help_arg() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;
    run_boat(["--help"], config_path)
        .success()
        .stdout(predicates::str::contains("Usage").or(predicates::str::contains("USAGE")));
    Ok(())
}

#[test]
fn test_help_subcommand_short_alias() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;
    run_boat(["h"], config_path)
        .success()
        .stdout(predicates::str::contains("Usage").or(predicates::str::contains("USAGE")));
    Ok(())
}

#[test]
fn test_version_arg() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;
    run_boat(["--version"], config_path)
        .success()
        .stdout(predicates::str::contains("boat"));
    Ok(())
}

#[test]
fn test_unknown_subcommand_fails() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;
    run_boat(["definitely-not-a-command"], config_path)
        .failure()
        .stderr(
            predicates::str::contains("error")
                .or(predicates::str::contains("not a valid subcommand")),
        );
    Ok(())
}
