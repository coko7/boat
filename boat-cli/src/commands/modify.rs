use anyhow::{Result, bail};
use boat_lib::repository::activities_repository as activities;
use log::info;
use rusqlite::Connection;

use crate::{cli, config::Configuration, utils};

pub fn modify(
    config: &Configuration,
    conn: &mut Connection,
    args: &cli::ModifyActivityArgs,
) -> Result<()> {
    let Ok(old) = activities::get_by_id(conn, args.id) else {
        info!("cannot modify because ID is invalid: {}", args.id);
        bail!(utils::display::activity_id_does_not_exist(args.id));
    };

    let ask_for_confirmation = utils::common::resolve_tri_state(
        args.confirm,
        args.no_confirm,
        config.commands.modify.confirm,
    );
    info!("ask user for confirmation? {ask_for_confirmation}");

    let prompt_msg = format!(
        "are you sure you want to modify activity #{} \"{}\"?",
        args.id, old.name
    );
    if ask_for_confirmation && !utils::common::prompt_for_confirmation(&prompt_msg, false)? {
        println!("user aborted the operation, activity was not modified");
        return Ok(());
    }

    info!("about to modify activity: {old:?}");
    activities::update(
        conn,
        args.id,
        args.update.name.as_deref(),
        args.update.description.as_deref(),
        args.update.tags.as_deref(),
    )?;

    let modified = activities::get_by_id(conn, args.id)?;
    info!("modified activity: {modified:?}");
    println!("{}", utils::display::modified_activity_msg(&modified));

    Ok(())
}
