use anyhow::{Result, bail};
use boat_lib::repository::activities_repository as activities;
use log::info;
use rusqlite::Connection;

use crate::{cli, config::Configuration, utils};

pub fn cancel_current(
    config: &Configuration,
    conn: &mut Connection,
    args: &cli::CancelActivityArgs,
) -> Result<()> {
    let ask_for_confirmation = utils::common::resolve_tri_state(
        args.confirm,
        args.no_confirm,
        config.commands.cancel.confirm,
    );
    info!("ask user for confirmation? {ask_for_confirmation}");

    match activities::get_current_ongoing(conn)? {
        Some(current) => {
            let prompt_msg = format!(
                "are you sure you want to cancel activity #{} \"{}\"?",
                current.id, current.name
            );
            if ask_for_confirmation && !utils::common::prompt_for_confirmation(&prompt_msg, false)?
            {
                println!("user aborted the operation, activity was not cancelled");
                return Ok(());
            }

            info!("cancelling current activity: {current:?}");
            activities::cancel_current(conn)?;
            println!("{}", utils::display::cancelled_activity_msg(&current));
        }
        None => {
            bail!(utils::display::no_current_act_msg())
        }
    }
    Ok(())
}
