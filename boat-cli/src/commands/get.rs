use anyhow::{Context, Result, bail};
use boat_lib::repository::activities_repository as activities;
use log::info;
use rusqlite::Connection;

use crate::{
    cli,
    config::Configuration,
    models::{activity::SimpleActivity, activity_log::PrintableActivityLog},
    utils,
};

pub fn get_current(
    config: &Configuration,
    conn: &Connection,
    args: &cli::PrintActivityArgs,
) -> Result<()> {
    match activities::get_current_ongoing(conn)? {
        Some(current) => {
            info!("got current activity: {current:?}");
            let ongoing_log = current
                .logs
                .iter()
                .find(|l| l.ends_at.is_none())
                .context("there should be an ongoing log")?
                .clone();

            info!("converting to simple act log");
            let simp_act = SimpleActivity::from_db_activity(&current);
            let activity_log = PrintableActivityLog::from_activity_and_log(&simp_act, &ongoing_log);

            if args.use_json_format {
                let json = serde_json::to_string(&activity_log)?;
                println!("{json}");
                return Ok(());
            }

            println!("{}", utils::display::current_activity_msg(&current)?);
        }
        None => bail!(utils::display::no_current_act_msg()),
    }

    Ok(())
}
