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

use anyhow::Context;
use anyhow::Result;
use clap::Parser;

use std::process;

use crate::game::Forward;
use crate::game::GameModule;
use crate::game::Information;
use crate::game::mnk;
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

fn main() -> Result<()> {
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
    match args.target {
        GameModule::ZeroBy => {
            let mut session = zero_by::Session::new(args.variant)?;
            if args.forward {
                let input = stdin_lines()
                    .context("Failed to read STDIN history input.")?;

                session
                    .forward(input)
                    .context("Failed to forward state with history input.")?
            }

            session
                .solve(args.mode)
                .context(format!(
                    "Failed solver execution for {}.",
                    zero_by::Session::info().name
                ))?
        },
        GameModule::Mnk => {
            let mut session = mnk::Session::new(args.variant)?;
            if args.forward {
                let input = stdin_lines()
                    .context("Failed to read STDIN history input.")?;

                session
                    .forward(input)
                    .context("Failed to forward state with history input.")?
            }

            session
                .solve(args.mode)
                .context(format!(
                    "Failed solver execution for {}.",
                    mnk::Session::info().name
                ))?
        },
    }
    Ok(())
}

fn info(args: InfoArgs) -> Result<()> {
    let data = match args.target {
        GameModule::ZeroBy => zero_by::Session::info(),
        GameModule::Mnk => mnk::Session::info(),
    };
    interface::cli::format_and_output_game_attributes(
        data,
        args.attributes,
        args.output,
    )?;
    Ok(())
}
