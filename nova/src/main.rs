#![warn(missing_docs)]
#![allow(unused_variables)]
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

use crate::execution::*;
use crate::interfaces::terminal::cli::*;

/* MODULES */

mod analyzers;
mod databases;
mod errors;
mod execution;
mod games;
mod interfaces;
mod models;
mod solvers;
mod utils;

/* PROGRAM ENTRY */

fn main()
{
    let cli = Cli::parse();
    match &cli.command {
        Commands::Tui(args) => tui(args, cli.quiet),
        Commands::Info(args) => info(args, cli.quiet),
        Commands::Solve(args) => solve(args, cli.quiet),
        Commands::Analyze(args) => analyze(args, cli.quiet),
    };
    process::exit(exitcode::OK);
}

/* SUBCOMMAND EXECUTORS */

fn tui(args: &TuiArgs, quiet: bool)
{
    todo!()
}

fn analyze(args: &AnalyzeArgs, quiet: bool)
{
    utils::confirm_potential_overwrite(args.yes, args.mode);
    todo!()
}

fn solve(args: &SolveArgs, quiet: bool)
{
    utils::confirm_potential_overwrite(args.yes, args.mode);
    let record = solving::solve_by_name(args, quiet);
    if let Some(output) = utils::format_record(&record, args.output) {
        if !quiet {
            println!("{}", output)
        }
    }
}

fn info(args: &InfoArgs, quiet: bool)
{
    if !quiet {
        listing::print_game_info(args.target, args.output)
    }
}
