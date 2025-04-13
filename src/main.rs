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

use anyhow::Result;
use clap::Parser;

use std::process;

use crate::interface::cli::*;
use crate::game::Information;
use crate::game::GameModule;

/* MODULES */

#[cfg(test)]
mod test;

mod interface;
mod solver;
mod game;
mod util;

/* PROGRAM ENTRY */

fn main() -> Result<()> {
    dotenv::dotenv()?;
    let cli = Cli::parse();
    let res = match cli.command {
        Commands::Info(args) => info(args),
        Commands::Build(args) => build(args),
    };
    if res.is_err() && cli.quiet {
        process::exit(exitcode::USAGE)
    }
    res
}

/* SUBCOMMAND EXECUTORS */

fn build(args: BuildArgs) -> Result<()> {
    todo!()
}

fn info(args: InfoArgs) -> Result<()> {
    let data = match args.target {
        GameModule::ZeroBy => game::zero_by::Session::info(),
        GameModule::Crossteaser => todo!(),
    };
    interface::cli::format_and_output_game_attributes(
        data,
        args.attributes,
        args.output,
    )?;
    Ok(())
}

