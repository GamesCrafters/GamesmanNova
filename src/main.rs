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

use std::{env, process};

use anyhow::{Context, Result};
use clap::Parser;

use crate::database::engine::sled::SledDatabase;
use crate::database::Persistent;
use crate::game::model::GameModule;
use crate::game::{Forward, Information};
use crate::interface::standard::cli::*;

/* MODULES */

mod interface;
mod database;
mod solver;
mod game;
mod util;

#[cfg(test)]
mod test;

/* PROGRAM ENTRY */

fn main() -> Result<()> {
    let cli = Cli::parse();
    let res = match cli.command {
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

fn query(args: QueryArgs) -> Result<()> {
    todo!()
}

fn solve(args: SolveArgs) -> Result<()> {
    interface::standard::confirm_potential_overwrite(args.yes, args.mode);
    let db_path = env::current_dir()?;
    let mut db = SledDatabase::new(&db_path)?;
    match args.target {
        GameModule::ZeroBy => {
            let mut session = game::zero_by::Session::new(args.variant)
                .context("Failed to initialize zero-by game session.")?;

            if args.forward {
                let history = interface::standard::stdin_lines()
                    .context("Failed to read input lines from STDIN.")?;

                session
                    .forward(history)
                    .context("Failed to forward game state.")?;
            }

            session
                .solve(args.mode, args.solution, &mut db)
                .context("Failed to execute solving algorithm.")?
        },
    }
    Ok(())
}

fn info(args: InfoArgs) -> Result<()> {
    let data = match args.target {
        GameModule::ZeroBy => game::zero_by::Session::info(),
    };
    interface::standard::format_and_output_game_attributes(
        data,
        args.attributes,
        args.output,
    )?;
    Ok(())
}
