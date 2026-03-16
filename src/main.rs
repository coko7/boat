use clap::Parser;
use log::LevelFilter;

use crate::cli::Cli;

mod activity;
mod cli;
mod data;
mod utils;

fn main() {
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_module("boat", LevelFilter::Debug)
        .filter_level(args.verbose.log_level_filter())
        .init();

    match &args.command {
        cli::Commands::Start { activity, category } => start_activity(activity, category),
        cli::Commands::List {
            show_all,
            show_current,
            show_categories,
        } => todo!(),
        cli::Commands::Resume(activity_id_selector) => todo!(),
        cli::Commands::Cancel(activity_id_selector) => todo!(),
        cli::Commands::Finish(activity_id_selector) => todo!(),
    }
}

fn start_activity(name: &str, category: &Option<String>) {
    println!("starting {} with: {:?}", name, category)
}
