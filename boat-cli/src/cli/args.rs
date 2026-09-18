use boat_lib::repository::Id;
use clap::ColorChoice;
use clap::Parser;
use clap::{ArgAction, Args, Subcommand, ValueEnum};
use serde::Deserialize;
use serde::Serialize;

use crate::cli::PeriodInput;

#[derive(Parser, Debug)]
#[command(
    name = "boat",
    version,
    author = "Made by @coko7 <contact@coko7.fr>",
    color = ColorChoice::Auto,
    about = "Basic Opinionated Activity Tracker",
    help_template = "{name} {version}\n\n{about}\n\n{usage-heading}\n{usage}\n\n{all-args}\n\n{author}"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Subcommand, Debug)]
#[command(rename_all = "kebab-case")]
pub enum Commands {
    /// Create a new activity
    #[command(alias = "n", alias = "create")]
    New(CreateActivityArgs),

    /// Start/resume an activity
    #[command(
        alias = "s",
        alias = "st",
        alias = "sail",
        alias = "continue",
        alias = "resume"
    )]
    Start(StartActivityArgs),

    /// Cancel the current activity
    #[command(alias = "c", alias = "can")]
    Cancel(CancelActivityArgs),

    /// Pause/stop the current activity
    #[command(alias = "p", alias = "stop")]
    Pause,

    /// Modify an activity
    #[command(alias = "m", alias = "mod")]
    Modify(ModifyActivityArgs),

    /// Edit activity logs as text in an external editor
    #[command(alias = "e", alias = "ed")]
    Edit(EditLogsArgs),

    /// Delete an activity
    #[command(
        alias = "d",
        alias = "del",
        alias = "rm",
        alias = "rem",
        alias = "remove"
    )]
    Delete(DeleteActivityArgs),

    /// Get the current activity
    #[command(alias = "g")]
    Get(PrintActivityArgs),

    /// List activity logs
    #[command(alias = "l", alias = "ls")]
    List(FilterActivitiesArgs),

    /// Show activity summaries
    #[command(alias = "r", alias = "rep")]
    Report(FilterActivitiesArgs),

    /// Generate a default boat config and output to stdout
    #[command(alias = "i")]
    Init,

    // /// Query boat objects
    // #[command(alias = "q")]
    // Query {
    //     #[command(subcommand)]
    //     command: QuerySubcommand,
    // },

    // This is ONLY way I could find to use the 'h' short alias for help.
    #[command(alias = "h", hide = true)]
    HelpExtension,
    // Verify the activity data and report eventual issues
    // #[command(alias = "v", alias = "verif")]
    // Verify {},

    // Query the different objects: activities, logs, tags
    // #[command(alias = "q")]
    // Query {},

    // Display a report with statistics about activities
    // #[command(alias = "r", alias = "rep")]
    // Report {},
    // ^^^ or maybe export 'x' ???
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, ValueEnum, Default, PartialEq, Eq)]
pub enum GroupBy {
    #[value(name = "none", alias = "no")]
    #[default]
    #[serde(rename = "none")]
    None,

    #[value(name = "day", alias = "d")]
    #[serde(rename = "day")]
    Day,

    #[value(name = "week", alias = "wk", alias = "w")]
    #[serde(rename = "week")]
    Week,

    #[value(name = "month", alias = "mo", alias = "m")]
    #[serde(rename = "month")]
    Month,

    #[value(name = "year", alias = "y")]
    #[serde(rename = "year")]
    Year,
}

impl std::fmt::Display for GroupBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            GroupBy::None => "none",
            GroupBy::Day => "day",
            GroupBy::Week => "week",
            GroupBy::Month => "month",
            GroupBy::Year => "year",
        };
        write!(f, "{str}")
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum SortBy {
    #[value(name = "none", alias = "no")]
    #[default]
    None,
}

#[derive(Args, Debug)]
pub struct FilterActivitiesArgs {
    /// Restrict matches to a given period: today, yesterday... (--period help to see all options)
    #[arg(short = 'p', long = "period")]
    pub period: Option<PeriodInput>,

    /// Specify how entries should be grouped
    #[arg(short = 'g', long = "group-by")]
    pub group_by: Option<GroupBy>,

    // /// Specify how entries should be sorted
    // #[arg(short = 's', long = "sort-by")]
    // pub sort_by: SortInput,
    /// Filter out entries that do not have all of the specified tags
    #[arg(short = 't', long = "filter-by-tags", value_name = "TAGS", value_delimiter = ',', action = ArgAction::Append)]
    pub filter_by_tags: Option<Vec<String>>,

    /// Select and order which fields are shown in the table (comma-separated)
    #[arg(short = 'f', long = "fields", value_name = "FIELDS", value_delimiter = ',', action = ArgAction::Append)]
    pub fields: Option<Vec<String>>,

    /// Output in JSON format
    #[arg(short = 'j', long = "json")]
    pub use_json_format: bool,
}

#[derive(Args, Debug, Default)]
#[group(multiple = false)]
pub struct PrintActivityArgs {
    /// Output in JSON
    #[arg(short = 'j', long = "json")]
    pub use_json_format: bool,
}

#[derive(Args, Debug)]
pub struct StartActivityArgs {
    /// ID of an existing activity or name for a new activity
    pub activity_handle: String,
}

#[derive(Args, Debug)]
pub struct SelectActivityArgs {
    /// ID of the activity
    pub activity_id: Id,
}

#[derive(Args, Debug)]
pub struct DeleteActivityArgs {
    /// ID of the activity to delete
    pub id: Id,

    /// Asks for confirmation before deleting the activity
    #[arg(short = 'c', long = "confirm", action = ArgAction::SetTrue, conflicts_with = "no_confirm")]
    pub confirm: bool,

    /// Skip the confirmation when deleting the activity
    #[arg(short = 'C', long = "no-confirm", action = ArgAction::SetTrue, conflicts_with = "confirm")]
    pub no_confirm: bool,
}

#[derive(Args, Debug)]
pub struct CancelActivityArgs {
    /// Asks for confirmation before cancelling the current activity
    #[arg(short = 'c', long = "confirm", action = ArgAction::SetTrue, conflicts_with = "no_confirm")]
    pub confirm: bool,

    /// Skip the confirmation when cancelling the current activity
    #[arg(short = 'C', long = "no-confirm", action = ArgAction::SetTrue, conflicts_with = "confirm")]
    pub no_confirm: bool,
}

#[derive(Args, Debug)]
pub struct CreateActivityArgs {
    /// Name of the activity
    pub name: String,

    /// ID of the parent activity
    #[arg(short, long)]
    pub description: Option<String>,

    /// List of tags to apply to the activity
    #[arg(short, long, value_delimiter = ',', action = ArgAction::Append)]
    pub tags: Vec<String>,

    /// Start the new activity automatically after creation
    #[arg(short = 's', long = "start-now", action = ArgAction::SetTrue, conflicts_with = "no_auto_start")]
    pub auto_start: bool,

    /// Prevent the new activity from starting automatically
    #[arg(short = 'S', long = "no-start-now", action = ArgAction::SetTrue, conflicts_with = "auto_start")]
    pub no_auto_start: bool,

    /// Output in JSON
    #[arg(short = 'j', long = "json")]
    pub use_json_format: bool,
}

#[derive(Args, Debug)]
pub struct ModifyActivityArgs {
    /// ID of the activity to edit
    pub id: Id,

    #[clap(flatten)]
    pub update: UpdateGroup,

    /// Asks for confirmation before applying changes
    #[arg(short = 'c', long = "confirm", action = ArgAction::SetTrue, conflicts_with = "no_confirm")]
    pub confirm: bool,

    /// Skip the confirmation before applying changes
    #[arg(short = 'C', long = "no-confirm", action = ArgAction::SetTrue, conflicts_with = "confirm")]
    pub no_confirm: bool,
}

#[derive(Args, Debug)]
#[group(required = true)]
pub struct UpdateGroup {
    /// New name for the activity
    #[arg(short = 'n', long = "name")]
    pub name: Option<String>,

    /// New description for the activity
    #[arg(short, long)]
    pub description: Option<String>,

    /// New list of tags to use for the activity
    #[arg(short, long, value_delimiter = ',', action = ArgAction::Append, num_args(0..))]
    pub tags: Option<Vec<String>>,
}

#[derive(Args, Debug)]
pub struct EditLogsArgs {
    /// Restrict to entries matching the given <PERIOD>
    #[arg(
        short = 'p',
        long = "period",
        help = "Period: day|d, week|w, month|m, year|y, <date>, or <start>..<end>"
    )]
    pub period: Option<PeriodInput>,

    /// Specify the editor program to use
    #[arg(short = 'e', long = "editor")]
    pub editor: Option<String>,

    /// Include instruction comments in the editable file
    #[arg(short = 'i', long = "with-instructions", alias = "with-instr", action = ArgAction::SetTrue, conflicts_with = "hide_instructions")]
    pub show_instructions: bool,

    /// Do not include instruction comments in the editable file
    #[arg(short = 'I', long = "no-instructions", alias = "no-instr", action = ArgAction::SetTrue, conflicts_with = "show_instructions")]
    pub hide_instructions: bool,

    /// Include activity definitions comments in the editable file
    #[arg(short = 'd', long = "with-activity-definitions", alias = "with-act-defs", alias = "with-defs", action = ArgAction::SetTrue, conflicts_with = "hide_activity_definitions")]
    pub show_activity_definitions: bool,

    /// Do not include activity definitions comments in the editable file
    #[arg(short = 'D', long = "no-activity-definitions", alias = "no-act-defs", alias = "no-defs", action = ArgAction::SetTrue, conflicts_with = "show_activity_definitions")]
    pub hide_activity_definitions: bool,

    /// Asks for confirmation before applying changes
    #[arg(short = 'c', long = "confirm", action = ArgAction::SetTrue, conflicts_with = "no_confirm")]
    pub confirm: bool,

    /// Skip the confirmation before applying changes
    #[arg(short = 'C', long = "no-confirm", action = ArgAction::SetTrue, conflicts_with = "confirm")]
    pub no_confirm: bool,
}
