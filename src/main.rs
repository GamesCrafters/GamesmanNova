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
use game::Variable;

use std::process;

use crate::game::GameModule;
use crate::game::Information;
use crate::game::crossteaser;
use crate::game::zero_by;
use crate::interface::cli::*;

/* MODULES */

#[cfg(test)]
mod test;

mod interface;
mod solver;
mod game;
mod util;

/* PROGRAM ENTRY */

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let res = match cli.command {
        Commands::Info(args) => info(args),
        Commands::Build(args) => build(args).await,
    };
    if res.is_err() && cli.quiet {
        process::exit(exitcode::USAGE)
    }
    res
}

/* SUBCOMMAND EXECUTORS */

async fn build(args: BuildArgs) -> Result<()> {
    util::prepare().await?;
    match args.target {
        GameModule::Crossteaser => {
            let session = if let Some(variant) = args.variant {
                crossteaser::Session::variant(variant)?;
            } else {
                crossteaser::Session::default();
            };
        },
        GameModule::ZeroBy => {
            let session = if let Some(variant) = args.variant {
                zero_by::Session::variant(variant)?;
            } else {
                zero_by::Session::default();
            };
        },
    }
    Ok(())
}

fn info(args: InfoArgs) -> Result<()> {
    let data = match args.target {
        GameModule::Crossteaser => crossteaser::Session::info(),
        GameModule::ZeroBy => zero_by::Session::info(),
    };
    interface::cli::format_and_output_game_attributes(
        data,
        args.attributes,
        args.output,
    )?;
    Ok(())
}
