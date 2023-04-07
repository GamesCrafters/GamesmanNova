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
use interfaces::terminal::cli::*;

/* Standard library */
use std::process;

/* PROGRAM ENTRY */

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Tui(args) => {
            tui(args);
        }
        Commands::Solve(args) => {
            solve(args);
        }
        Commands::Analyze(args) => {
            analyze(args);
        }
    }
    process::exit(exitcode::OK);
}

/* SUBCOMMAND EXECUTORS */

fn tui(args: &TuiArgs) {}

fn solve(args: &SolveArgs) {}

fn analyze(args: &AnalyzeArgs) {}
