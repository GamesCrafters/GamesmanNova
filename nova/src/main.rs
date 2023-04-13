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

use clap::Parser;
use core::Value;
use errors::{user::UserError, NovaError};
use interfaces::terminal::cli::*;
use serde_json::json;
use solving::solve_by_names;
use std::process;

/* MODULES */

mod analyzing;
mod errors;
mod interfacing;
mod solving;
mod utils;

/* PROGRAM ENTRY */

fn main() {
    let cli = Cli::parse();
    let result: Result<(), UserError>;
    match &cli.command {
        Commands::Tui(args) => {
            result = tui(args, cli.quiet, cli.verbose, cli.yes);
        }
        Commands::Solve(args) => {
            result = solve(args, cli.quiet, cli.verbose, cli.yes);
        }
        Commands::Analyze(args) => {
            result = analyze(args, cli.quiet, cli.verbose, cli.yes);
        }
    }
    if let Err(e) = result {
        println!("{}", e);
        process::exit(exitcode::USAGE);
    }
    process::exit(exitcode::OK);
}

/* SUBCOMMAND EXECUTORS */

/// Spawns a terminal user interface
fn tui(args: &TuiArgs, q: bool, v: bool, y: bool) -> Result<(), UserError> {
    todo!()
}

fn solve(args: &SolveArgs, q: bool, v: bool, y: bool) -> Result<(), UserError> {
    let value = solve_by_names(&args.target, args.solver.clone())?;
    format_print(value, args);
    Ok(())
}

fn analyze(args: &AnalyzeArgs, q: bool, v: bool, y: bool) -> Result<(), UserError> {
    todo!()
}

fn format_print(value: Value, args: &SolveArgs) {
    let value_str: &str;
    let remoteness: u32;
    match value {
        Value::Lose(rem) => {
            value_str = "lose";
            remoteness = rem;
        }
        Value::Tie(rem) => {
            value_str = "tie";
            remoteness = rem;
        }
        Value::Win(rem) => {
            value_str = "win";
            remoteness = rem;
        }
    }
    if let Some(format) = args.output {
        match format {
            Output::Formatted => {
                println!(
                    "\nYou solved {}. The game is a {} for the first player in {} moves.\n",
                    args.target, value_str, remoteness
                );
            }
            Output::Json => {
                let json = json!({
                    "value": value_str,
                    "remoteness": remoteness
                });
                println!("{}", json);
            }
        }
    } else {
        println!("{} in {}", value_str, remoteness);
    }
}
