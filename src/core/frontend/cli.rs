//! # Command Line Applications
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use serde_json::Map;
use serde_json::Value;

use std::io::BufRead;

use crate::types::frontend::GAME_ATTRIBUTES;
use crate::types::frontend::GameAttribute;
use crate::types::frontend::IOMode;
use crate::types::frontend::InfoFormat;
use crate::types::game::GameData;
use crate::types::game::GameModule;

/* CLI DECLARATIONS */

/// Nova generates datasets of sequential game states and associated features.
#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct Cli {
    /* REQUIRED COMMANDS */
    /// Available subcommands for the main 'nova' command.
    #[command(subcommand)]
    pub command: Commands,

    /* DEFAULTS PROVIDED */
    /// Send no output to STDOUT.
    #[arg(short, long, group = "output")]
    pub quiet: bool,
}

/// Subcommand choices, specified as `nova <subcommand>`.
#[derive(Subcommand)]
pub enum Commands {
    /// Build a dataset associated with a sequential game.
    Build(BuildArgs),

    /// Print information about the system's offering(s).
    Info(InfoArgs),
}

/// Arguments to the `nova build` subcommand.
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

/// Arguments to the `nova info` subcommand.
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

/* STANDARD INPUT */

/// Parses STDIN into a line-by-line vector of its contents without any form of
/// sanitation or formatting.
pub fn stdin_lines() -> Result<Vec<String>> {
    std::io::stdin()
        .lock()
        .lines()
        .map(|l| l.map_err(|e| anyhow!(e)))
        .collect()
}

/* STANDARD OUTPUT */

/// Collects the attributes specified in `attrs` from the provided game `data`
/// into a specific `format`, and prints them to STDOUT. If `attrs` is `None`,
/// all possible game attributes are sent to STDOUT.
pub fn format_and_output_game_attributes(
    data: GameData,
    attrs: Vec<GameAttribute>,
    format: InfoFormat,
) -> Result<()> {
    let attrs = (!attrs.is_empty())
        .then_some(attrs)
        .unwrap_or(GAME_ATTRIBUTES.to_vec());

    let out = aggregate_and_format_attributes(data, attrs, format)
        .context("Failed format specified game data attributes.")?;

    print!("{out}");
    Ok(())
}

/* HELPER FUNCTIONS */

/// Collects the attributes specified in `attr` from the provided game `data`
/// to a single string in a specific `format`.
pub fn aggregate_and_format_attributes(
    data: GameData,
    attrs: Vec<GameAttribute>,
    format: InfoFormat,
) -> Result<String> {
    match format {
        InfoFormat::Legible => {
            let mut output = String::new();
            attrs.iter().for_each(|&a| {
                output += &format!("\t{a}:\n{}\n\n", data.find(a))
            });
            Ok(output)
        },
        InfoFormat::Json => {
            let mut map = Map::new();
            attrs.iter().for_each(|&a| {
                map.insert(a.to_string(), Value::String(data.find(a).into()));
            });
            serde_json::to_string(&map)
                .context("Failed to generate JSON object from game data.")
        },
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn no_duplicates_in_game_attrs_list() {
        let mut attrs = GAME_ATTRIBUTES.to_vec();
        let s1 = attrs.len();
        attrs.sort();
        attrs.dedup();
        let s2 = attrs.len();
        assert_eq!(s1, s2);
    }
}
