//! # Solving Execution Module
//!
//! This module contains handling behavior for all `nova list` requests.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::errors::NovaError;
use crate::interfaces::terminal::cli::*;
use crate::utils::check_game_exists;
use crate::{
    games,
    games::{Game, GameInformation},
};
use serde_json::json;

/// Prints the formatted game information according to the output specified in
/// `args`.
pub fn printf_game_info(args: &InfoArgs, game: &String) -> Result<(), NovaError> {
    check_game_exists(game)?;
    let target = &game[0..];
    let session = get_session::generate_match!("src/games/")(None);
    let info: GameInformation = session.info();
    if let Some(format) = args.output {
        match format {
            Output::Extra => {
                println!("\tGame:\n{}\n", info.name);
                println!("\tAuthor:\n{}\n", info.author);
                println!("\tDescription:\n{}\n", info.about);
                println!("\tVariant Protocol:\n{}\n", info.variant_protocol);
                println!("\tVariant Default:\n{}\n", info.variant_default);
                println!("\tVariant Pattern:\n{}\n", info.variant_pattern);
            }
            Output::Json => {
                println!(
                    "{}",
                    json!({
                        "game": info.name,
                        "author": info.author,
                        "about": info.about.replace("\n", " "),
                        "variant-protocol": info.variant_protocol.replace("\n", " "),
                        "variant-default": info.variant_default,
                        "variant-pattern": info.variant_pattern,
                    })
                );
            }
        }
    } else {
        println!("\t{}:\n{}\n", game, info.about);
    }
    Ok(())
}

/// Prints the formatted game list according to the output specified in `args`.
pub fn printf_game_list(args: &InfoArgs) {
    if let Some(format) = args.output {
        match format {
            Output::Extra => {
                println!("Here are the game targets available:\n");
                for (i, game) in games::LIST.iter().enumerate() {
                    println!("{}. {}", i, game);
                }
            }
            Output::Json => {
                let mut contents: String = String::new();
                for game in games::LIST {
                    contents += &format!("\"{}\",\n", game);
                }
                let json = json!({ "games": [contents] });
                println!("{}", json);
            }
        }
    } else {
        for game in games::LIST {
            println!("{}", game);
        }
    }
}
