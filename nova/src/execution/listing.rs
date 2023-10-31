//! # Solving Execution Module
//!
//! This module contains handling behavior for all `nova list` requests.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::games::{Game, GameData};
use crate::interfaces::terminal::cli::*;
use crate::interfaces::{find_game, GameModule};
use serde_json::json;

/// Prints the formatted game information according to a specified output
/// format. Game information is provided by game implementations.
pub fn print_game_info(game: &GameModule, format: Option<OutputFormat>)
{
    let info: GameData = find_game(game, None).info();
    if let Some(format) = format {
        match format {
            OutputFormat::Extra => {
                println!("\tGame:\n{}\n", info.name);
                println!("\tAuthor:\n{}\n", info.author);
                println!("\tDescription:\n{}\n", info.about);
                println!("\tCategory:\n{}\n", info.category);
                println!("\tVariant Protocol:\n{}\n", info.variant_protocol);
                println!("\tVariant Default:\n{}\n", info.variant_default);
                println!("\tVariant Pattern:\n{}\n", info.variant_pattern);
            }
            OutputFormat::Json => {
                println!(
                    "{}",
                    json!({
                        "game": info.name,
                        "author": info.author,
                        "about": info.about,
                        "category": info.category,
                        "variant-protocol": info.variant_protocol,
                        "variant-default": info.variant_default,
                        "variant-pattern": info.variant_pattern,
                    })
                );
            }
            OutputFormat::None => {}
        }
    } else {
        println!("\t{}:\n{}\n", game, info.about);
    }
    Ok(())
}
