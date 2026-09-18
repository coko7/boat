use anyhow::Result;
use boat_lib::repository::activities_repository as activities;
use chrono::Local;
use log::info;
use rusqlite::Connection;

use crate::{config::Configuration, utils};

pub fn pause_current(config: &Configuration, conn: &mut Connection) -> Result<()> {
    match activities::get_current_ongoing(conn)? {
        Some(current) => {
            info!("ongoing activity: {current:?}");
            activities::stop_current(conn)?;
            println!(
                "{}",
                utils::display::paused_activity_msg(&current, Local::now())?
            );
        }
        None => {
            info!("no ongoing activity");
            println!("{}", utils::display::no_current_act_msg());
        }
    };

    Ok(())
}
