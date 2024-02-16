//! # Solving Execution Module
//!
//! This module contains handling behavior for all `nova list` requests.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::error::NovaError;
use crate::game::GameData;
use crate::interface::terminal::cli::*;
use crate::interface::{find_game, GameModule};
use serde_json::json;
use std::fmt::Display;

/* API */

/// Prints the formatted game information according to a specified output
/// format. Game information is provided by game implementations.
pub fn print_game_info(
    game: GameModule,
    format: OutputFormat,
) -> Result<(), NovaError> {
    find_game(game, None, None)?
        .info()
        .print(format);
    Ok(())
}

/* HELPER FUNCTIONS */

impl GameData<'_> {
    fn print(&self, format: OutputFormat) {
        match format {
            OutputFormat::Extra => {
                let content = format!(
                    "\tGame:\n{}\n\n\tAuthor:\n{}\n\n\tDescription:\n{}\n\n\t\
                    Variant Protocol:\n{}\n\n\tVariant Default:\n{}\n\n\t\
                    Variant Pattern:\n{}\n\n\tState Protocol:\n{}\n\n\tState \
                    Default:\n{}\n\n\tState Pattern:\n{}\n",
                    self.name,
                    self.authors,
                    self.about,
                    self.variant_protocol,
                    self.variant_default,
                    self.variant_pattern,
                    self.state_protocol,
                    self.state_default,
                    self.state_pattern
                );
                println!("{}", content);
            },
            OutputFormat::Json => {
                let content = json!({
                    "game": self.name,
                    "author": self.authors,
                    "about": self.about,
                    "variant-protocol": self.variant_protocol,
                    "variant-default": self.variant_default,
                    "variant-pattern": self.variant_pattern,
                    "state-protocol": self.state_protocol,
                    "state-default": self.state_default,
                    "state-pattern": self.state_pattern,
                });
                println!("{}", content);
            },
            OutputFormat::None => (),
        }
    }
}

impl Display for GameData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\tGame:\n{}\n\n\tAuthor:\n{}\n\n\tDescription:\n{}\n\n\tVariant \
            Protocol:\n{}\n\n\tVariant Default:\n{}\n\n\tVariant Pattern:\n{}\
            \n\n\tState Protocol:\n{}\n\n\tState Default:\n{}\n\n\tState \
            Pattern:\n{}\n",
            self.name,
            self.authors,
            self.about,
            self.variant_protocol,
            self.variant_default,
            self.variant_pattern,
            self.state_protocol,
            self.state_default,
            self.state_pattern
        )
    }
}
