#![warn(deprecated)]
//! # Nova
//!
//! TODO

use anyhow::Result;
use clap::Parser;

use std::process;

use crate::frontend::cli;
use crate::game::GameModule;
use crate::game::traits::Information;
use crate::game::zero_by;

/* MODULES */

#[cfg(test)]
pub mod developer;
pub mod scheduler;
pub mod frontend;
pub mod database;
pub mod macros;
pub mod error;
pub mod game;

/* PROGRAM ENTRY */

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let res = match cli.command {
        cli::Commands::Info(args) => info(args),
        cli::Commands::Build(args) => build(args),
    };

    if res.is_err() && cli.quiet {
        process::exit(exitcode::USAGE)
    }

    res
}

/* SUBCOMMAND EXECUTORS */

fn build(args: cli::BuildArgs) -> Result<()> {
    Ok(())
}

fn info(args: cli::InfoArgs) -> Result<()> {
    let data = match args.target {
        GameModule::ZeroBy => zero_by::Session::info(),
    };
    cli::format_and_output_game_attributes(data, args.attributes, args.output)?;
    Ok(())
}
