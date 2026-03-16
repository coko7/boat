use clap::{Args, Parser, Subcommand};

use crate::data::ActivityId;

#[derive(Parser)]
#[command(
    name = "boat",
    version,
    about = "Basic Opinionated Activity Tracker",
    author = "coko7",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum Commands {
    #[command(alias = "s")]
    Start {
        /// Name of the activity to start
        activity: String,

        /// Category for this activity
        #[arg(short, long, value_name = "NAME")]
        category: Option<String>,
    },

    #[command(alias = "r")]
    Resume(ActivityIdSelector),

    #[command(alias = "c")]
    Cancel(ActivityIdSelector),

    #[command(alias = "f")]
    Finish(ActivityIdSelector),

    #[command(alias = "l", alias = "ls")]
    List {
        #[arg(short = 'a', long = "all", conflicts_with_all = ["show_current","show_categories"])]
        show_all: bool,

        #[arg(short = 'c', long = "current", conflicts_with_all = ["show_all","show_categories"])]
        show_current: bool,

        #[arg(short = 'C', long = "category", conflicts_with_all = ["show_all","show_current"])]
        show_categories: bool,
    },
}

#[derive(Args)]
pub struct ActivityIdSelector {
    /// ID of the activity
    activity_id: ActivityId,
}
