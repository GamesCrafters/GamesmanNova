#![warn(missing_docs, deprecated)]
//! # Nova
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use clap::Parser;

use std::process;

use crate::application::frontend::cli;
use crate::interface::game::Forward;
use crate::interface::game::Information;
use crate::model::game::GameModule;
use crate::model::game::zero_by;

/* MODULES */

mod application {
    #[cfg(test)]
    pub mod developer;
    pub mod frontend {
        pub mod cli;
    }

    pub mod database {
        pub mod sqlite;
        pub mod sled;
    }

    pub mod solver {
        pub mod scheduler;
        pub mod routines;
    }
}

mod implementation {
    pub mod frontend;
    pub mod database;
    pub mod record;
    pub mod solver;
    pub mod error;
    pub mod game;
}

mod interface {
    pub mod database;
    pub mod game;
}

mod model {
    #[cfg(test)]
    pub mod developer;
    pub mod frontend;
    pub mod database;
    pub mod record;
    pub mod solver;
    pub mod error;
    pub mod game;
}

mod macros;

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
            let mut session = zero_by::Session::new(args.variant)?;
            if args.forward {
                let input = cli::stdin_lines()
                    .context("Failed to read STDIN history input.")?;

                session
                    .forward(input)
                    .context("Failed to forward state with history input.")?
            }

            session
                .solve(args.mode)
                .context("Failed solver execution for crossteaser.")?
        },
    }
    Ok(())
}

fn info(args: cli::InfoArgs) -> Result<()> {
    let data = match args.target {
        GameModule::ZeroBy => zero_by::Session::info(),
    };
    cli::format_and_output_game_attributes(data, args.attributes, args.output)?;
    Ok(())
}
