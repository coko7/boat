# ⛵ boat

`boat` - A **B**asic **O**pinionated **A**ctivity **T**racker, inspired by [bartib](https://github.com/nikolassv/bartib).

![](./docs/boat-cli.png)

<p align="center">
    <a href="https://crates.io/crates/boat-cli"><img src="https://img.shields.io/crates/v/boat-cli.svg" alt="Crates info"></a>
    <a href="LICENSE"><img src="https://img.shields.io/github/license/coko7/boat-cli?color=blue" alt="License: MIT"></a>
    <img src="https://img.shields.io/github/languages/top/coko7/boat-cli?color=orange" alt="Rust">
    <a href="https://github.com/coko7/boat-cli/actions/workflows/rust.yml"><img src="https://github.com/coko7/boat-cli/actions/workflows/rust.yml/badge.svg" alt="Tests"></a>
</p>

Like its name implies, `boat` allows you to track the time
you spend on everyday tasks.

It has mainly been designed to be easy to embed in custom bash scripts
so that you can augment it with fuzzy-finding.
That said, if you plan to use the CLI directly (without external scripts),
it also benefits from a [variety of handy aliases](#-usage).

`boat` stores its data in a SQLite database file which is kept
in the config directory by default (`.config/boat/boat.db`).

This repository contains only the code for the command line application.
It relies on [`boat-lib`](https://github.com/coko7/boat-lib) for core functions.

## Contents

- [🚀 Demo](#-demo)
- [🤔 Why was this tool created?](#-why-was-this-tool-created)
- [🛠️ Installation](#%EF%B8%8F-installation)
  - [Install with a bundled version of SQLite](#install-with-a-bundled-version-of-sqlite)
- [⚙️ Configuration](#%EF%B8%8F-configuration)
- [✨ Usage](#-usage)
  - [New](#new)
  - [Start](#start)
  - [Cancel](#cancel)
  - [Pause](#pause)
  - [Modify](#modify)
  - [Edit](#edit)
  - [Delete](#delete)
  - [Get](#get)
  - [List](#list)
  - [Report](#report)
  - [Init](#init)
- [🔮 Alternatives to boat](#-alternatives-to-boat)
- [🧠 (mostly) Brain made](#-mostly-brain-made)

## 🚀 Demo

![demo](https://github.com/user-attachments/assets/007809ee-fda9-4848-bc1c-ccd0131a3616)

> [!TIP]
> The `mp4` version of the demo is available [here](https://github.com/user-attachments/assets/919cb800-1d65-4c3f-bbae-a3195db3b3f6)

## 🤔 Why was this tool created?

The [`bartib`](https://github.com/nikolassv/bartib) CLI is what inspired me to create `boat`.
It's a feature-full tool that I used for a while,
but I found it quite limiting for my usage due to its [lack of support for machine-readable output](https://github.com/nikolassv/bartib/pull/26).
And that's it. All I wanted was an activity tracker that I could combine easily with [`jq`](https://github.com/jqlang/jq) and so I decided to make my own tool.

## 🛠️ Installation

The easiest way to install is through [crates.io](https://crates.io/crates/boat-cli):

```sh
cargo install boat-cli
```

The other option is to build from source:

```sh
git clone https://github.com/coko7/boat-cli.git
cd boat-cli
cargo build --release
```

### Install with a bundled version of SQLite

If you are on Windows or you do not have SQLite on your system, you can install `boat` with the `bundled-sqlite` feature.
This will increase the compilation time but guarantees a working out-of-the-box experience.

When using cargo:

```sh
cargo install boat-cli --features bundled-sqlite
```

When building from source:

```sh
git clone https://github.com/coko7/boat-cli.git
cd boat-cli
cargo build --release --features bundled-sqlite
```

## ⚙️ Configuration

By default, `boat` will create a configuration file in one of the following directories:

- 🐧 **Linux:** `/home/<user>/.config/boat/config.toml`
- 🪟 **Windows:** `C:\Users\<user>\AppData\Roaming\boat\config.toml`
- 🍎 **MacOS:** `/Users/<user>/Library/Application Support/boat/config.toml`

It will also store the SQLite database file `boat.db` in the same directory (unless specified otherwise in config):

```toml
database_path = "/home/<user>/.config/boat/boat.db"
```

You can override the default configuration file path by setting the `BOAT_CONFIG` environment variable.
Here is the full default configuration:

```toml
database_path = "/home/<user>/.config/boat/boat.db"
period = "all"
format = "plain"

[commands.new]
auto_start = false

[commands.start]
quick_start = true

[commands.cancel]
confirm = true

[commands.edit]
show_instructions = true
show_activity_definitions = true
confirm = true

[commands.delete]
confirm = true

[commands.list]
period = "month"
group_by = "day"

[commands.report]
period = "day"
```

## ✨ Usage

If you have ever used [`bartib`](https://github.com/nikolassv/bartib), then `boat` is going to feel very familiar.
Try `boat help` for a quick list of commands:

```help
boat 0.9.0

Basic Opinionated Activity Tracker

Usage:
boat [OPTIONS] <COMMAND>

Commands:
  new     Create a new activity
  start   Start/resume an activity
  cancel  Cancel the current activity
  pause   Pause/stop the current activity
  modify  Modify an activity
  edit    Edit activity logs as text in an external editor
  delete  Delete an activity
  get     Get the current activity
  list    List activity logs
  report  Show activity summaries
  init    Generate a default boat config and output to stdout
  help    Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Increase logging verbosity
  -q, --quiet...    Decrease logging verbosity
  -h, --help        Print help
  -V, --version     Print version

Made by @coko7 <contact@coko7.fr>
```

> [!TIP]
> `boat` comes bundled with many command aliases:
>
> - new: `n`, `new`, `create`
> - start: `s`, `st`, `start`, `sail`, `continue`, `resume`
> - cancel: `c`, `can`, `cancel`
> - pause: `p`, `pause`, `stop`
> - modify: `m`, `mod`, `modify`
> - edit: `e`, `ed`, `edit`
> - delete: `d`, `del`, `delete`, `rm`, `rem`, `remove`
> - get: `g`, `get`
> - list: `l`, `ls`, `list`
> - report: `r`, `rep`, `report`
> - help: `h`, `help`, `-h`, `--help`
>
> Prefer using the full length command names in scripts as they are more explicit and unlikely to be changed (unlike shorter aliases).

I really wanted to have each command start with a different character so that I could assign a single-char alias to all of them.
That explains why some of the commands do not use a more fitting keyword.

Like `stop` would have been a better command than `pause` but since it shares the same starting character as the `start` command, I could not use it.
Maybe I will drop this in the future, let's see.

_I have included some fallback in case you type `stop`/`remove` instead of `pause`/`delete` 👀_

Below is a breakdown of all the available commands and their options:

<details id="new">

  <summary>New: Create a new activity</summary>

You can use the `new` command to create a new activity, with an optional **description** and **tags** (comma-separated list). If you want to start the activity immediately, you can include the `-s`/`--start-now` flag:

```console
$ boat new 'take down prod' -s
created new #51 "take down prod"
started #51 "take down prod" at 20:42
```

If you want to process the output in scripts, you can use the `-j`/`--json` flag:

```console
$ boat new 'fetch gruvbox wallpaper' \
  -d 'fetching the latest gruvbox wallpapers online' \
  -t leisure,ricing -j
{
  "id": 52,
  "name": "fetch gruvbox wallpaper",
  "description": "fetching the latest gruvbox wallpapers online",
  "duration": 0,
  "ongoing": false,
  "tags": [
    "leisure",
    "ricing"
  ]
}
```

Full list of options:

```help
Create a new activity

Usage: boat new [OPTIONS] <NAME>

Arguments:
  <NAME>  Name of the activity

Options:
  -d, --description <DESCRIPTION>  ID of the parent activity
  -t, --tags <TAGS>                List of tags to apply to the activity
  -s, --start-now                  Start the new activity automatically after creation
  -S, --no-start-now               Prevent the new activity from starting automatically
  -j, --json                       Output in JSON
  -v, --verbose...                 Increase logging verbosity
  -q, --quiet...                   Decrease logging verbosity
  -h, --help                       Print help
```

</details>

<details id="start">

  <summary>Start: Start/resume an activity</summary>

The `start` command may be used to start tracking time for an existing activity.
It expects an **activity handle** which may be either:

- the ID of an existing activity (e.g. `51`)
- the name of a new activity to create and start immediately (e.g. `take down prod`)

Since boat uses whole integers as activity IDs, if you provide an activity handle that cannot be parsed as an integer, boat will assume that you want to create a new activity with the provided name and start it immediately.

**Examples:**

With an existing activity ID:

```console
$ boat start 52
paused #51 "take down prod" at 20:55
started #52 "push to master" at 20:55
```

With a new activity name:

```console
$ boat start 'work on presenterm slides'
created new #55 "work on presenterm slides"
started #55 "work on presenterm slides" at 20:58
```

> [!TIP]
> Make sure you have `quick_start` enabled for the `start` command in your config file if you want to be able to start new activities with this command. Otherwise, you will just get an error.

Full list of options:

```help
Start/resume an activity

Usage: boat start [OPTIONS] <ACTIVITY_HANDLE>

Arguments:
  <ACTIVITY_HANDLE>  ID of an existing activity or name for a new activity

Options:
  -v, --verbose...  Increase logging verbosity
  -q, --quiet...    Decrease logging verbosity
  -h, --help        Print help
```

</details>

<details id="cancel">

  <summary>Cancel: Cancel the current activity</summary>

If you started an activity by mistake, you can use the `cancel` command to quickly revert it.
You can control the confirmation behavior with the `-c`/`--confirm` and `-C`/`--no-confirm` flags or the `confirm` option in the config file.

Full list of options:

```help
Cancel the current activity

Usage: boat cancel [OPTIONS]

Options:
  -c, --confirm     Asks for confirmation before cancelling the current activity
  -C, --no-confirm  Skip the confirmation when cancelling the current activity
  -v, --verbose...  Increase logging verbosity
  -q, --quiet...    Decrease logging verbosity
  -h, --help        Print help
```

</details>

<details id="pause">

  <summary>Pause: Pause/stop the current activity</summary>

To stop tracking time for the current activity, you can use the `pause` command:

```console
$ boat pause
paused #55 "work on presenterm slides" at 21:02
```

Full list of options:

```help
Pause/stop the current activity

Usage: boat pause [OPTIONS]

Options:
  -v, --verbose...  Increase logging verbosity
  -q, --quiet...    Decrease logging verbosity
  -h, --help        Print help
```

</details>

<details id="modify">

  <summary>Modify: Pause/stop the current activity</summary>

You can modify the **name**, **description**, and **tags** of an existing activity with the `modify` command:

```console
$ boat new tests
created new #56 "tests"

$ boat modify
boat mod 56 -n 'write tests' -d 'write some smoke tests' -t test
are you sure you want to modify activity #56 "tests"? yes
modified #56 "write tests"
```

Full list of options:

```help
Modify an activity

Usage: boat modify [OPTIONS] <--name <NAME>|--description <DESCRIPTION>|--tags [<TAGS>...]> <ID>

Arguments:
  <ID>  ID of the activity to edit

Options:
  -n, --name <NAME>                New name for the activity
  -d, --description <DESCRIPTION>  New description for the activity
  -t, --tags [<TAGS>...]           New list of tags to use for the activity
  -c, --confirm                    Asks for confirmation before applying changes
  -C, --no-confirm                 Skip the confirmation before applying changes
  -v, --verbose...                 Increase logging verbosity
  -q, --quiet...                   Decrease logging verbosity
  -h, --help                       Print help
```

</details>

<details id="edit">

  <summary>Edit: Edit activity logs as text in an external editor</summary>

If you want to make adjustments to the activity logs (start/end tracking times), you can make use of the `edit` command. Calling the `edit` command will open the list of logs in CSV format in your default `$EDITOR`. You can tweak the `start`/`end` times for multiple logs in there.
After saving the file and quitting the file, you will be able to preview the changes to be applied and either proceed with the update or discard it.

You may provide a period to edit only a subset of the logs with the `-p`/`--period` flag. If no period is provided, it defaults to whatever is in your config file.

For example, let's say you want to explicitly edit the logs for today, you can do:

```console
boat edit -p today
```

This will open up a file with your list of activities:

```csv
# This is a CSV export of your activities and logs.
# Lines starting with '#' are comments and should not be modified.
# Activity definitions are included for reference but are not meant to be edited.
# You may only edit activity logs here.
#
# Activity definitions:
# | ID | Name | Description | Tags |
# | -- | ---- | ----------- | ---- |
# | 51 | take down prod |  |  |
# | 52 | push to master |  |  |
# | 55 | work on presenterm slides |  |  |


# Below are your activity logs. You can edit the start and end times here.
# If you want to mark the latest activity as ongoing, simply remove the end time (leave it blank) for that log.
# Please keep the activity_id and log_id unchanged to avoid breaking the data.
#
# Logs (activity_id,log_id,starts_at,ends_at):
# ===== EDIT DATA BELOW =====
51,119,2026-04-24 20:42,2026-04-24 20:55
52,120,2026-04-24 20:55,2026-04-24 20:58
55,121,2026-04-24 20:58,2026-04-24 21:02
```

The file includes comments to help you understand how you can edit the logs in there.
If you start being familiar with the format, you can disable the instructions and activity definitions in the file with the `-I`/`--no-instructions` and `-D`/`--no-activity-definitions` flags.
This behavior can also be configured in the config file with the `show_instructions` and `show_activity_definitions` options under the `edit` command.

After saving and quitting the file, you will get a preview of the changes to be applied:

```console
$ boat edit -p today
Detected changes:
Log ID 121: starts_at: 2026-04-24 18:58:05 UTC (no change), ends_at: Some(2026-04-24T19:02:58Z) -> 2026-04-24T19:03:00Z
You are about to update 1 log entries. Do you want to proceed? yes
successfully updated 1 log entries
```

Full list of options:

```help
Edit activity logs as text in an external editor

Usage: boat edit [OPTIONS]

Options:
  -p, --period <PERIOD>            Period: day|d, week|w, month|m, year|y, <date>, or <start>..<end>
  -e, --editor <EDITOR>            Specify the editor program to use
  -i, --with-instructions          Include instruction comments in the editable file
  -I, --no-instructions            Do not include instruction comments in the editable file
  -d, --with-activity-definitions  Include activity definitions comments in the editable file
  -D, --no-activity-definitions    Do not include activity definitions comments in the editable file
  -c, --confirm                    Asks for confirmation before applying changes
  -C, --no-confirm                 Skip the confirmation before applying changes
  -v, --verbose...                 Increase logging verbosity
  -q, --quiet...                   Decrease logging verbosity
  -h, --help                       Print help
```

</details>

<details id="delete">

  <summary>Delete: Delete an activity</summary>

You may delete an activity with the `delete` command. This will **permanently delete the activity and all its logs**, so be careful with it.

Let's say you started an activity by mistake:

```console
boat new 'go back to using Windows'
```

I don't know what brought you to do this point, but it's okay, you can fix it.
All you need to do is to delete the activity and all its related logs will disappear just like that:

```
$ boat delete 57
are you sure you want to delete activity #57 "go back to using Windows"? yes
deleted #57 "go back to using Windows"
```

😌 _Phewww_, now nobody will ever know about this dark chapter of your life.

Full list of options:

```help
Delete an activity

Usage: boat delete [OPTIONS] <ID>

Arguments:
  <ID>  ID of the activity to delete

Options:
  -c, --confirm     Asks for confirmation before deleting the activity
  -C, --no-confirm  Skip the confirmation when deleting the activity
  -v, --verbose...  Increase logging verbosity
  -q, --quiet...    Decrease logging verbosity
  -h, --help        Print help
```

</details>

<details id="get">

  <summary>Get: Get the current activity</summary>

Acquiring information about the current activity is really simple. All you need to do is to use the `get` command:

```console
$ boat get
current: #55 "work on presenterm slides": 21:24 -> Now (10 seconds)
```

This command also supports JSON output with the `-j`/`--json` flag:

```console
$ boat g -j
{
  "log": {
    "starts_at": "2026-04-24T21:24:18+02:00",
    "ends_at": null
  },
  "activity": {
    "id": 55,
    "name": "work on presenterm slides",
    "description": null,
    "tags": []
  }
}
```

Full list of options:

```help
Get the current activity

Usage: boat get [OPTIONS]

Options:
  -j, --json        Output in JSON
  -v, --verbose...  Increase logging verbosity
  -q, --quiet...    Decrease logging verbosity
  -h, --help        Print help
```

</details>

<details id="list">

  <summary>List: List activity logs</summary>

The `list` command can be used to get an overview of all the activities you have been tracking.
It comes with a lot of options to filter and group the results. You can also output the list in JSON format for further processing in scripts.

The period argument is very extensive and allows you to filter logs using preset periods (e.g. `today`, `yesterday`, `last-week`, `this-month`, `last-month`, etc.), exact dates (e.g. `2026-04-24`) or date ranges (e.g. `2026-04-01..2026-04-24`).

Here is a complete overview of all the available values:

```help
- Period presets:
  - today|tod|td
  - yesterday|ytd|yd
  - this-week|tw|twk|wk
  - last-week|lw|lwk|yesterweek|yw|ywk
  - this-month|tm|tmo|mo
  - last-month|lm|lmo|yestermonth|ym|ymo
  - all-time|all
- Exact date: YYYY-MM-DD
- Range: YYYY-MM-DD..YYYY-MM-DD
```

Full list of options:

```console
List activity logs

Usage: boat list [OPTIONS]

Options:
  -p, --period <PERIOD>        Restrict matches to a given period: today, yesterday... (--period help to see all options)
  -g, --group-by <GROUP_BY>    Specify how entries should be grouped [possible values: none, day, week, month, year]
  -t, --filter-by-tags <TAGS>  Filter out entries that do not have all of the specified tags
  -j, --json                   Output in JSON format
  -v, --verbose...             Increase logging verbosity
  -q, --quiet...               Decrease logging verbosity
  -h, --help                   Print help
```

</details>

<details id="report">

  <summary>Report: Show activity summaries</summary>

The `report` command can be used to get a summary of the total time spent on your activities.
It uses the same filtering options as the `list` command but **does not support grouping yet.**

Full list of options:

```help
Show activity summaries

Usage: boat report [OPTIONS]

Options:
  -p, --period <PERIOD>        Restrict matches to a given period: today, yesterday... (--period help to see all options)
  -g, --group-by <GROUP_BY>    Specify how entries should be grouped [possible values: none, day, week, month, year]
  -t, --filter-by-tags <TAGS>  Filter out entries that do not have all of the specified tags
  -j, --json                   Output in JSON format
  -v, --verbose...             Increase logging verbosity
  -q, --quiet...               Decrease logging verbosity
  -h, --help                   Print help
```

</details>

<details id="init">

  <summary>Init: Generate a default boat config and output to stdout</summary>

If, at anytime, you need to get a copy of the default configuration file, you can make use of the `init` command and the full TOML representation will be printed in your terminal for you to copy-paste:

```console
$ boat init
database_path = "/home/<user>/.config/boat/boat.db"

[commands.new]
auto_start = false

[commands.start]
quick_start = true

[commands.cancel]
confirm = true

[commands.modify]
confirm = true

[commands.edit]
show_instructions = true
show_activity_definitions = true
confirm = true

[commands.delete]
confirm = true

[commands.list]

[commands.report]
```

Full list of options:

```help
Generate a default boat config and output to stdout

Usage: boat init [OPTIONS]

Options:
  -v, --verbose...  Increase logging verbosity
  -q, --quiet...    Decrease logging verbosity
  -h, --help        Print help
```

</details>

## 🔮 Alternatives to boat

Hey. I made `boat` to solve my own very specific problems but I don't expect it to be a perfect fit for everyone. If you are looking for similar tools, I got you:

- [`bartib`](https://github.com/nikolassv/bartib): A simple timetracker for the command line. It saves a log of all tracked activities as a plaintext file and allows you to create flexible reports.
- [`zeit`](https://github.com/mrusme/zeit): Zeit, erfassen. A command line tool for tracking time.

## 🧠 (mostly) Brain made

**This project was NOT vibe-coded BUT AI is still involved in some parts of it.**

I think generating big portions of code using AI can be justified in some contexts, and I am not opposed to it if done well. But, I care about this project too much to vibe code it :)

Still, **AI is used** in this project and here's where:

- **Generating tests:** Because it's something I always skip so I would rather have some AI generated tests than none at all.
- **Micro-improvements:** I have used AI as an advisor to improve some bits of code here and there. Big refactors or new features are done by my hand though. _(This is why this tool breaks so often I guess 💀)_

<a href="https://brainmade.org/">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://brainmade.org/white-logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://brainmade.org/black-logo.svg">
  <img alt="brainmade" src="https://brainmade.org/white-logo.svg">
</picture>
</a>
