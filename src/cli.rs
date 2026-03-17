use clap::{ArgAction, Args, Parser, Subcommand};

use crate::activity::ActId;

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
    /// Create a new activity
    #[command(alias = "n")]
    New(CreateActivityArgs),

    /// Start/resume an activity
    #[command(alias = "s", alias = "st")]
    Start(SelectActivityArgs),

    /// Pause/stop the current activity
    #[command(alias = "p")]
    Pause,

    /// Modify an activity
    #[command(alias = "m", alias = "mod")]
    Modify(ModifyActivityArgs),

    /// Delete an activity
    #[command(alias = "d", alias = "del")]
    Delete(SelectActivityArgs),

    /// Get the current activity
    #[command(alias = "g")]
    Get(PrintActivityArgs),

    /// List activities
    #[command(alias = "l", alias = "ls")]
    List(ListActivityArgs),

    /// Edit the raw content of activity files
    #[command(alias = "e", alias = "ed")]
    Edit(EditFilesArgs),

    /// Display a report with statistics about activities
    #[command(alias = "r", alias = "rep")]
    Report {},
    // ^^^ or maybe export 'x' ???
}

#[derive(Args)]
pub struct ListActivityArgs {
    #[arg(short = 'a', long = "all", conflicts_with_all = ["show_current","show_categories"])]
    show_all: bool,

    #[arg(short = 'c', long = "current", conflicts_with_all = ["show_all","show_categories"])]
    show_current: bool,

    #[arg(short = 'C', long = "category", conflicts_with_all = ["show_all","show_current"])]
    show_categories: bool,
}

#[derive(Args)]
#[group(multiple = false)]
pub struct PrintActivityArgs {
    /// Output in pretty format
    #[arg(short = 'p', long = "pretty")]
    use_pretty_format: bool,

    /// Output in JSON
    #[arg(short = 'j', long = "json")]
    use_json_format: bool,
}

impl Default for PrintActivityArgs {
    fn default() -> Self {
        Self {
            use_pretty_format: true,
            use_json_format: false,
        }
    }
}

#[derive(Args)]
pub struct SelectActivityArgs {
    /// ID of the activity
    activity_id: ActId,
}

#[derive(Args)]
pub struct CreateActivityArgs {
    /// Name of the activity
    activity: String,

    /// ID of the parent activity
    #[arg(short, long)]
    parent: Option<String>,

    /// List of tags to apply to the activity
    #[arg(short, long, value_delimiter = ',', action = ArgAction::Append)]
    tags: Vec<String>,

    /// Start the new activity automatically
    #[arg(short, long)]
    start: bool,
}

#[derive(Args)]
pub struct ModifyActivityArgs {
    /// ID of the activity to edit
    id: String,

    #[clap(flatten)]
    update: UpdateGroup,
}

#[derive(clap::Args)]
#[group(required = true)]
pub struct UpdateGroup {
    /// New name for the activity
    #[arg(short = 'n', long = "name")]
    name: Option<String>,

    /// New ID for the parent activity
    #[arg(short = 'p', long = "parent")]
    parent: Option<String>,

    /// New list of tags to use for the activity
    #[arg(short, long, value_delimiter = ',', action = ArgAction::Append)]
    tags: Option<Vec<String>>,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct EditFilesArgs {
    #[arg(short = 'd', long = "definitions", alias = "def")]
    edit_definitions: bool,

    #[arg(short = 'l', long = "logs")]
    edit_logs: bool,
}
