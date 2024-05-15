//! # Standard Interface Module
//!
//! This module defines the behavior of the project's standard (STDIN/STDOUT)
//! interfaces. The CLI module is the primary entry point for the program.
//!
//! #### Authorship
//! - Max Fierro, 7/5/2024

use anyhow::{anyhow, Context, Result};

use std::{io::BufRead, process};

use crate::interface::util;
use crate::interface::{GameAttribute, InfoFormat};
use crate::{game::GameData, interface::IOMode};

/* SPECIFIC INTERFACES */

pub mod cli;

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
    print!("{}", out);
    Ok(())
}
