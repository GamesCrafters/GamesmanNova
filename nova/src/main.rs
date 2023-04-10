#![warn(missing_docs)]

//! # GamesmanNova Executioner
//!
//! The module which aggregates the libraries provided in `core`, `games`, and
//! `interfaces` to provide an entry point to all the functionality of the
//! project.
//!
//! Instead of this project's modules having an emphasized many-to-many
//! relationship, greater weight is placed on making things fit into this
//! module as a centralized point.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

/* External crates */
use clap::Parser;

/* Internal crates */
use crate::utils::most_similar;
use core::{Value, archetypes::Game};
use errors::user::GameNotFoundError;
use games::IMPLEMENTED_GAMES;
use interfaces::terminal::cli::*;

/* Standard library */
use std::process;

/* MODULES */

mod errors;
mod utils;

/* PROGRAM ENTRY */

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Tui(args) => {
            tui(args);
        }
        Commands::Solve(args) => {
            if let Err(e) = solve(args) {
                println!("{}", e);
            }
        }
        Commands::Analyze(args) => {
            analyze(args);
        }
    }
    process::exit(exitcode::OK);
}

/* SUBCOMMAND EXECUTORS */

fn tui(args: &TuiArgs) {}

fn solve(args: &SolveArgs) -> Result<(), GameNotFoundError> {
    let value = solve_by_name(&args.target)?;
    match value {
        Value::Lose(rem) => { println!("Lose in {}", rem); },
        Value::Tie(rem) => { println!("Tie in {}", rem); },
        Value::Win(rem) => { println!("Win in {}", rem); }
    }
    Ok(())
}

fn analyze(args: &AnalyzeArgs) {}

/* HELPERS */

fn solve_by_name(name: &str) -> Result<Value, GameNotFoundError> {
    match name {
        "10-to-0-by-1-or-2" => {
            let sesh = games::ten_to_zero_by_one_or_two::Session::new();
            let solve = sesh.solvers()[0].1;
            Ok(solve(&sesh))
        },
        _ => {
            let not_found_error = GameNotFoundError {
                input: name.to_owned(),
                suggestion: most_similar(name, IMPLEMENTED_GAMES.to_vec()),
            };
            Err(not_found_error)
        }
    }
}
