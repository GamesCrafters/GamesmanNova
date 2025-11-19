#![warn(deprecated)]
//! # Nova
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use clap::Parser;

use std::process;

use crate::frontend::cli;
use crate::game::GameModule;
use crate::game::traits::Advance;
use crate::game::traits::Information;
use crate::game::traits::Variable;
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
    match args.target {
        GameModule::ZeroBy => {
            let mut game = zero_by::Session::variant(args.variant)?;
            if args.advance {
                let history = cli::stdin_lines()?;
                game.advance(history)
                    .context("Failed to forward game via history")?;
            }

            game.build(args.mode)?;
        },
    };

    Ok(())
}

fn info(args: cli::InfoArgs) -> Result<()> {
    let data = match args.target {
        GameModule::ZeroBy => zero_by::Session::info(),
    };

    let attrs = if !args.attributes.is_empty() {
        args.attributes
    } else {
        frontend::GAME_ATTRIBUTES.to_vec()
    };

    let out = cli::aggregate_and_format_attributes(data, attrs, args.output)
        .context("Failed format specified game data attributes.")?;

    print!("{out}");
    Ok(())
}
