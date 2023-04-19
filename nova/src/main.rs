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

use crate::core::Value;
use crate::errors::UserError;
use crate::interfaces::terminal::cli::*;
use crate::operations::*;

/* MODULES */

mod core;
mod errors;
mod games;
mod interfaces;
mod operations;
mod utils;

/* PROGRAM ENTRY */

fn main() {
    let cli = Cli::parse();
    let result: Result<(), UserError> = match &cli.command {
        Commands::Tui(args) => tui(args, cli.quiet),
        Commands::Solve(args) => solve(args, cli.quiet),
        Commands::Analyze(args) => analyze(args, cli.quiet),
        Commands::Info(args) => list(args, cli.quiet),
    };
    if let Err(e) = result {
        if !cli.quiet {
            println!("{}", e);
        }
        process::exit(exitcode::USAGE);
    }
    process::exit(exitcode::OK);
}

/* SUBCOMMAND EXECUTORS */

fn tui(args: &TuiArgs, quiet: bool) -> Result<(), UserError> {
    todo!()
}

fn analyze(args: &AnalyzeArgs, quiet: bool) -> Result<(), UserError> {
    todo!()
}

fn solve(args: &SolveArgs, quiet: bool) -> Result<(), UserError> {
    solving::confirm_potential_overwrite(args);
    let value = solving::solve_by_name(
        &args.target,
        &args.variant,
        &args.solver,
        args.read,
        args.write,
        quiet,
    )?;
    if !quiet {
        solving::printf_solve_result(value, args);
    }
    Ok(())
}

fn list(args: &InfoArgs, quiet: bool) -> Result<(), UserError> {
    if !quiet {
        if let Some(game) = &args.target {
            listing::printf_game_info(args, game)?;
        } else {
            listing::printf_game_list(args);
        }
    }
    Ok(())
}
