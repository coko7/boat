use clap::Parser;
use log::LevelFilter;

use crate::cli::Cli;

mod activity;
mod cli;
mod converter;
mod parser;
mod store;
mod utils;
mod validator;

fn main() {
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_module("boat", LevelFilter::Debug)
        .filter_level(args.verbose.log_level_filter())
        .init();

    match &args.command {
        cli::Commands::New(create_activity_args) => todo!(),
        cli::Commands::Start(select_activity_args) => todo!(),
        cli::Commands::Pause => todo!(),
        cli::Commands::Modify(modify_activity_args) => todo!(),
        cli::Commands::Delete(select_activity_args) => todo!(),
        cli::Commands::Get(print_activity_args) => todo!(),
        cli::Commands::List(list_activity_args) => todo!(),
        cli::Commands::Edit(edit_files_args) => todo!(),
        cli::Commands::Report {} => todo!(),
    }
}

fn start_activity(name: &str, category: &[String]) {
    println!("starting {} with: {:?}", name, category)
}
