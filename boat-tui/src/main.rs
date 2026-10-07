use anyhow::Result;
use std::process::ExitCode;

use crate::{app::App, config::Config};

mod app;
mod config;
mod form;
mod report;
mod ui;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let config = Config::load()?;
    let conn = boat_lib::utils::init_database(&config.boat.database_path)?;
    let mut app = App::new(config, conn)?;

    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}
