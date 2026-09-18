use anyhow::{Result, bail};
use boat_lib::repository::{Id, activities_repository as activities};
use chrono::Local;
use log::{info, warn};
use rusqlite::Connection;
use yansi::Paint;

use crate::{
    cli,
    commands::{self, pause_current},
    config::Configuration,
    utils,
};

pub fn start(
    config: &Configuration,
    conn: &mut Connection,
    args: &cli::StartActivityArgs,
) -> Result<()> {
    let activity_id = args.activity_handle.parse::<Id>();

    if config.commands.start.quick_start {
        info!("quick start enabled, allow to create a named activity and start it on the fly");

        if activity_id.is_err() {
            return commands::create::create(
                config,
                conn,
                &cli::CreateActivityArgs {
                    name: args.activity_handle.clone(),
                    description: None,
                    tags: vec![],
                    auto_start: true,
                    no_auto_start: false,
                    use_json_format: false,
                },
            );
        }
    }

    let activity_id = match activity_id {
        Ok(id) => id,
        Err(_) => {
            warn!(
                "cannot start because activity handle is not an integer '{}' and quick start is disabled",
                args.activity_handle
            );
            bail!(utils::display::invalid_activity_id_format(
                &args.activity_handle
            ));
        }
    };

    let Ok(to_start) = activities::get_by_id(conn, activity_id) else {
        info!("cannot start because ID is invalid: {}", activity_id);
        bail!(utils::display::activity_id_does_not_exist(activity_id));
    };

    if let Some(current) = activities::get_current_ongoing(conn)? {
        info!("ongoing activity: {current:?}");

        if current.id == activity_id {
            info!("not starting because same activity already ongoing");
            println!("{}", "activity already ongoing...".italic());
            return Ok(());
        }

        info!("pausing current...");
        pause_current(config, conn)?;
    }

    info!("about to start: {to_start:?}");
    activities::start(conn, activity_id)?;
    println!(
        "{}",
        utils::display::started_activity_msg(&to_start, Local::now())
    );

    Ok(())
}
