#![allow(unused_variables)]
#![warn(missing_docs)]
//! # Execution Module
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
use std::process;

use crate::errors::NovaError;
use crate::execution::*;
use crate::interfaces::terminal::cli::*;

/* MODULES */

mod analyzers;
mod database;
mod errors;
mod execution;
mod games;
mod interfaces;
mod models;
mod solvers;
mod utils;

/* PROGRAM ENTRY */

fn main() {
    let cli = Cli::parse();
    let ret = match &cli.command {
        Commands::Tui(args) => tui(args),
        Commands::Info(args) => info(args),
        Commands::Solve(args) => solve(args),
        Commands::Analyze(args) => analyze(args),
    };
    if let Err(e) = ret {
        if !cli.quiet {
            eprintln!("{}", e);
        }
        process::exit(exitcode::USAGE)
    }
    process::exit(exitcode::OK)
}

/* SUBCOMMAND EXECUTORS */

fn tui(args: &TuiArgs) -> Result<(), NovaError> {
    todo!()
}

fn analyze(args: &AnalyzeArgs) -> Result<(), NovaError> {
    utils::confirm_potential_overwrite(args.yes, args.mode);
    todo!()
}

fn solve(args: &SolveArgs) -> Result<(), NovaError> {
    utils::confirm_potential_overwrite(args.yes, args.mode);
    solving::solve_by_name(args)?;
    Ok(())
}

fn info(args: &InfoArgs) -> Result<(), NovaError> {
    informing::print_game_info(args.target, args.output);
    Ok(())
}
