use anyhow::Result;
use boat_lib::{models::activity::NewActivity, repository::activities_repository as activities};
use chrono::Local;
use log::info;
use rusqlite::Connection;

use crate::{
    cli,
    config::Configuration,
    models::activity::{PrintableActivity, SimpleActivity},
    utils,
};

pub fn create(
    config: &Configuration,
    conn: &mut Connection,
    args: &cli::CreateActivityArgs,
) -> Result<()> {
    let start_auto = utils::common::resolve_tri_state(
        args.auto_start,
        args.no_auto_start,
        config.commands.new.auto_start,
    );

    info!("start the new activity automatically? {start_auto}");
    let new = NewActivity {
        name: args.name.clone(),
        description: args.description.clone(),
        tags: args.tags.clone(),
    };

    info!("creating new activity: {new:?}");

    let created = activities::create(conn, new)?;
    if !args.use_json_format {
        println!("{}", utils::display::created_activity_msg(&created));
    }

    if start_auto {
        info!("activity auto_start is enabled");
        activities::start(conn, created.id)?;
        if !args.use_json_format {
            println!(
                "{}",
                utils::display::started_activity_msg(&created, Local::now())
            );
        }
    }

    let simp_act = SimpleActivity::from_db_activity(&created);
    let act = PrintableActivity::from_activity_and_logs(&simp_act, &created.logs);
    if args.use_json_format {
        let json = serde_json::to_string(&act)?;
        println!("{json}");
    }

    Ok(())
}
