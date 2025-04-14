//! # Command Line Module
//!
//! This module offers UNIX-like CLI tooling in order to facilitate scripting
//! and ergonomic use of GamesmanNova. This uses the
//! [clap](https://docs.rs/clap/latest/clap/) crate to provide standard
//! behavior, which is outlined in [this](https://clig.dev/) great guide.

use anyhow::{Context, Result, anyhow};
use clap::{Args, Parser, Subcommand};

use std::{io::BufRead, process};

use crate::game::GameModule;
use crate::interface::util;
use crate::interface::{GameAttribute, InfoFormat};
use crate::{game::GameData, interface::IOMode};

/* CLI DEFINITIONS */

/// TODO
#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct Cli {
    /* REQUIRED COMMANDS */
    /// Available subcommands for the main 'nova' command.
    #[command(subcommand)]
    pub command: Commands,

    /* DEFAULTS PROVIDED */
    /// Send no output to STDOUT during successful execution.
    #[arg(short, long, group = "output")]
    pub quiet: bool,
}

/// Subcommand choices, specified as `nova <subcommand>`.
#[derive(Subcommand)]
pub enum Commands {
    /// Build a dataset associated with an exploration game.
    Build(BuildArgs),

    /// Provides information about the system's offerings.
    Info(InfoArgs),
}

/* ARGUMENT AND OPTION DEFINITIONS */

/// TODO
#[derive(Args)]
pub struct BuildArgs {
    /* REQUIRED ARGUMENTS */
    /// Target game name.
    pub target: GameModule,

    /* OPTIONAL ARGUMENTS */
    /// Solve a specific variant of game.
    #[arg(short, long)]
    pub variant: Option<String>,

    /// Specify whether the solution should be fetched or re-generated.
    #[arg(short, long, default_value_t = IOMode::Constructive)]
    pub mode: IOMode,

    /// Compute solution starting after a state history read from STDIN.
    #[arg(short, long)]
    pub forward: bool,
}

/// TODO
#[derive(Args)]
pub struct InfoArgs {
    /// Specify the game to provide information about.
    pub target: GameModule,

    /// Specify which of the game's attributes to provide information about.
    #[arg(short, long, value_delimiter = ',', num_args(1..))]
    pub attributes: Vec<GameAttribute>,

    /* OPTIONAL ARGUMENTS */
    /// Format in which to send output to STDOUT.
    #[arg(short, long, default_value_t = InfoFormat::Legible)]
    pub output: InfoFormat,
}

/* STANDARD INPUT API */

/// Prompts the user to confirm their operation as appropriate according to the
/// arguments of the solve command. Only asks for confirmation for potentially
/// destructive operations.
pub fn confirm_potential_overwrite(yes: bool, mode: IOMode) {
    if match mode {
        IOMode::Overwrite => !yes,
        IOMode::Constructive => false,
    } {
        println!(
            "This may overwrite an existing solution database. Are you sure? \
            [y/n]: "
        );
        let mut yn: String = "".to_owned();
        while !["n", "N", "y", "Y"].contains(&&yn[..]) {
            yn = String::new();
            std::io::stdin()
                .read_line(&mut yn)
                .expect("Failed to read user confirmation.");
            yn = yn.trim().to_string();
        }
        if yn == "n" || yn == "N" {
            process::exit(exitcode::OK)
        }
    }
}

/// Parses STDIN into a line-by-line vector of its contents without any form of
/// sanitation or formatting.
pub fn stdin_lines() -> Result<Vec<String>> {
    std::io::stdin()
        .lock()
        .lines()
        .map(|l| l.map_err(|e| anyhow!(e)))
        .collect()
}

/* STANDARD OUTPUT API */

/// Collects the attributes specified in `attrs` from the provided game `data`
/// into a specific `format`, and prints them to STDOUT. If `attrs` is `None`,
/// all possible game attributes are sent to STDOUT.
pub fn format_and_output_game_attributes(
    data: GameData,
    attrs: Vec<GameAttribute>,
    format: InfoFormat,
) -> Result<()> {
    let out = if attrs.is_empty() {
        util::aggregate_and_format_all_attributes(data, format)
            .context("Failed to format game data attributes.")?
    } else {
        util::aggregate_and_format_attributes(data, attrs, format)
            .context("Failed format specified game data attributes.")?
    };
    print!("{out}");
    Ok(())
}
