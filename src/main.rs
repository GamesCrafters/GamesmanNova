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

use std::process;

use anyhow::Result;
use clap::Parser;

use crate::interface::standard::cli::*;
use crate::target::model::TargetModule;
use crate::target::Information;

/* MODULES */

mod interface;
mod database;
mod solver;
mod target;
mod util;

#[cfg(test)]
mod test;

/* PROGRAM ENTRY */

fn main() -> Result<()> {
    let cli = Cli::parse();
    let res = match cli.command {
        Commands::Info(args) => info(args),
        Commands::Extract(args) => extract(args),
        Commands::Frame(args) => frame(args),
    };
    if res.is_err() && cli.quiet {
        process::exit(exitcode::USAGE)
    }
    res
}

/* SUBCOMMAND EXECUTORS */

fn frame(args: FrameArgs) -> Result<()> {
    todo!()
}

fn extract(args: ExtractArgs) -> Result<()> {
    todo!()
}

fn info(args: InfoArgs) -> Result<()> {
    let data = match args.target {
        TargetModule::ZeroBy => target::game::zero_by::Session::info(),
    };
    interface::standard::format_and_output_game_attributes(
        data,
        args.attributes,
        args.output,
    )?;
    Ok(())
}
