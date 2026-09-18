use anyhow::Result;
use assert_cmd::Command;
use tempfile::TempDir;

pub fn cli_args_for_temp() -> Result<(TempDir, std::path::PathBuf)> {
    let tmp = TempDir::new()?;
    let db_path = tmp.path().join("boat.db");
    let config_path = tmp.path().join("boat_config.toml");

    let config_content = format!("database_path = {:?}\n", db_path.display())
        + r#"
period = "all"
format = "plain"

[commands.new]
auto_start = false

[commands.start]
quick_start = true

[commands.cancel]
confirm = true

[commands.modify]
confirm = true

[commands.edit]
show_instructions = true
show_activity_definitions = true
confirm = true

[commands.delete]
confirm = true

[commands.list]
group_by = "day"

[commands.report]
"#;

    std::fs::write(&config_path, config_content.as_bytes())?;
    Ok((tmp, config_path))
}

pub fn run_boat<I, S, P>(args: I, config_path: P) -> assert_cmd::assert::Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
    P: AsRef<std::ffi::OsStr>,
{
    Command::cargo_bin("boat")
        .unwrap()
        .env("BOAT_CONFIG", config_path)
        .args(args)
        .assert()
}
