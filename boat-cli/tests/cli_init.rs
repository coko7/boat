//! Integration tests for the `init` CLI command.
use anyhow::Result;

use crate::utils::{cli_args_for_temp, run_boat};

mod utils;

#[test]
fn init_outputs_valid_toml_with_database_path() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["init"], config_path)
        .success()
        .stdout(predicates::str::contains("database_path"));

    Ok(())
}

#[test]
fn init_works_without_existing_config_file() {
    // `init` must run before config loading, so it should succeed even when the
    // config file doesn't exist yet.
    assert_cmd::Command::cargo_bin("boat")
        .unwrap()
        .env("BOAT_CONFIG", "/tmp/boat_init_test_nonexistent.toml")
        .args(["init"])
        .assert()
        .success()
        .stdout(predicates::str::contains("database_path"));
}
