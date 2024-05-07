#![warn(missing_docs, deprecated)]
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

use std::process;

use anyhow::{Context, Result};
use clap::Parser;
use game::Variable;

use crate::interface::terminal::cli::*;
use crate::model::game::GameModule;

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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let res = match &cli.command {
        Commands::Info(args) => info(args),
        Commands::Solve(args) => solve(args),
        Commands::Query(args) => query(args),
    };
    if res.is_err() && cli.quiet {
        process::exit(exitcode::USAGE)
    }
    res
}

/* SUBCOMMAND EXECUTORS */

fn tui(args: &QueryArgs) -> Result<()> {
    todo!()
}

fn query(args: &QueryArgs) -> Result<()> {
    todo!()
}

fn solve(args: &SolveArgs) -> Result<()> {
    util::confirm_potential_overwrite(args.yes, args.mode);
    match args.target {
        GameModule::ZeroBy => {
            let session = game::zero_by::Session::new()
                .into_variant(args.variant.clone())
                .context("Failed to initialize zero-by game session.")?;
        },
    }
    Ok(())
}

fn info(args: &InfoArgs) -> Result<()> {
    Ok(())
}
