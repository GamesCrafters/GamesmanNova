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
use errors::UserError;
use games::IMPLEMENTED_GAMES;
use handling::solving::solve_by_name;
use interfaces::terminal::cli::*;
use serde_json::json;
use std::process;

/* MODULES */

mod errors;
mod handling;
mod utils;

/* PROGRAM ENTRY */

fn main() {
    let cli = Cli::parse();
    let result: Result<(), UserError>;
    match &cli.command {
        Commands::Tui(args) => {
            result = tui(args, cli.quiet, cli.yes);
        }
        Commands::Solve(args) => {
            result = solve(args, cli.quiet, cli.yes);
        }
        Commands::Analyze(args) => {
            result = analyze(args, cli.quiet, cli.yes);
        }
        Commands::List(args) => {
            result = list(args, cli.quiet);
        }
    }
    if let Err(e) = result {
        println!("{}", e);
        process::exit(exitcode::USAGE);
    }
    process::exit(exitcode::OK);
}

/* SUBCOMMAND EXECUTORS */

fn tui(args: &TuiArgs, q: bool, y: bool) -> Result<(), UserError> {
    todo!()
}

fn solve(args: &SolveArgs, q: bool, y: bool) -> Result<(), UserError> {
    let value = solve_by_name(
        &args.target,
        args.variant.clone(),
        args.solver.clone(),
        args.read,
        args.write,
    )?;
    if !q {
        format_print_value(value, args);
    }
    Ok(())
}

fn analyze(args: &AnalyzeArgs, q: bool, y: bool) -> Result<(), UserError> {
    todo!()
}

fn list(args: &ListArgs, q: bool) -> Result<(), UserError> {
    if !q {
        format_print_list(args);
    }
    Ok(())
}

/* HELPER FUNCTIONS */

fn format_print_value(value: Value, args: &SolveArgs) {
    let value_str: &str;
    let remoteness: u8;
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

fn format_print_list(args: &ListArgs) {
    if let Some(format) = args.output {
        match format {
            Output::Formatted => {
                println!("Here are the game targets available:\n");
                for (i, game) in IMPLEMENTED_GAMES.iter().enumerate() {
                    println!("{}. {}\n", i, game);
                }
            }
            Output::Json => {
                let mut contents: String = String::new();
                for game in IMPLEMENTED_GAMES {
                    contents += &format!("\"{}\",\n", game);
                }
                let json = json!({ "games": [contents] });
                println!("{}", json);
            }
        }
    } else {
        for game in IMPLEMENTED_GAMES {
            println!("{}\n", game);
        }
    }
}
