use anyhow::{Result, bail};
use boat_lib::repository::activities_repository as activities;
use log::info;
use rusqlite::Connection;

use crate::{cli, config::Configuration, utils};

pub fn delete(
    config: &Configuration,
    conn: &mut Connection,
    args: &cli::DeleteActivityArgs,
) -> Result<()> {
    let ask_for_confirmation = utils::common::resolve_tri_state(
        args.confirm,
        args.no_confirm,
        config.commands.delete.confirm,
    );
    info!("ask user for confirmation? {ask_for_confirmation}");

    let Ok(to_delete) = activities::get_by_id(conn, args.id) else {
        info!("cannot delete because ID is invalid: {}", args.id);
        bail!(utils::display::activity_id_does_not_exist(args.id));
    };

    let prompt_msg = format!(
        "are you sure you want to delete activity #{} \"{}\"?",
        to_delete.id, to_delete.name
    );
    if ask_for_confirmation && !utils::common::prompt_for_confirmation(&prompt_msg, false)? {
        println!("user aborted the operation, activity was not deleted");
        return Ok(());
    }

    info!("deleting activity: {to_delete:?}");
    activities::delete(conn, args.id)?;
    println!("{}", utils::display::deleted_activity_msg(&to_delete));

    Ok(())
}
