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
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use anyhow::Result;
use clap::Parser;

use std::process;

use crate::interface::terminal::cli::*;

/* MODULES */

mod interface;
mod database;
mod solver;
mod model;
mod game;
mod util;

#[cfg(test)]
mod test;

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

fn tui(args: &TuiArgs) -> Result<()> {
    todo!()
}

fn analyze(args: &AnalyzeArgs) -> Result<()> {
    todo!()
}

fn solve(args: &SolveArgs) -> Result<()> {
    util::confirm_potential_overwrite(args.yes, args.mode);
    let game = util::find_game(
        args.target,
        args.variant.to_owned(),
        args.from.to_owned(),
    )?;
    game.solve(args.mode, args.solver)?;
    Ok(())
}

fn info(args: &InfoArgs) -> Result<()> {
    util::print_game_info(args.target, args.output)?;
    Ok(())
}
