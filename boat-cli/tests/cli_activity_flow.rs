//! Test basic activity CRUD and flow in the CLI
use anyhow::Result;

use crate::utils::{cli_args_for_temp, run_boat};

mod utils;

#[test]
fn can_create_start_pause_list_activity() -> Result<()> {
    let (_tmp, config_path) = cli_args_for_temp()?;

    run_boat(["new", "TestTask"], &config_path).success();

    // boat start <ID: always 1 for first activity>
    run_boat(["start", "1"], &config_path).success();

    // boat pause
    run_boat(["pause"], &config_path)
        .success()
        .stdout(predicates::str::contains("paused"));

    // boat list --json, just check output contains the activity name 'TestTask'
    run_boat(["list", "--json"], &config_path)
        .success()
        .stdout(predicates::str::contains("TestTask"));

    Ok(())
}
