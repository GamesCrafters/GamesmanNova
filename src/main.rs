#![warn(deprecated)]
//! # Nova
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use clap::Parser;

use std::process;
use std::sync::Arc;

use crate::database::rocksdb::init_rocksdb;
use crate::database::storage::RocksDBStorage;
use crate::frontend::IOMode;
use crate::frontend::cli;
use crate::game::GameModule;
use crate::game::traits::Advance;
use crate::game::traits::Implicit;
use crate::game::traits::Information;
use crate::game::traits::Partition;
use crate::game::traits::Variable;
use crate::game::zero_by;
use crate::scheduler::*;

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
    let history = if args.advance { Some(cli::stdin_lines()?) } else { None };

    match args.target {
        GameModule::ZeroBy => {
            let ruleset = zero_by::Ruleset::variant(args.variant)?;
            build_game::<_, zero_by::Record>(ruleset, args.mode, history)
        },
    }
}

fn build_game<G, R>(
    mut ruleset: G,
    mode: IOMode,
    history: Option<Vec<String>>,
) -> Result<()>
where
    G: Variable + Implicit + Partition + Advance + Clone + Send + 'static,
    R: TryFrom<Vec<u8>, Error = anyhow::Error>
        + Into<Vec<u8>>
        + Default
        + Clone
        + Send
        + Sync
        + 'static,
{
    if let Some(hist) = history {
        ruleset
            .advance(hist)
            .context("Failed to forward game via history")?;
    }

    let storage = {
        let db = init_rocksdb(mode, ruleset.name())?;
        Arc::new(RocksDBStorage::<R>::new(db))
    };

    let task = {
        let executable = ForwardTaskBuilder::<_, R>::default()
            .source(ruleset.source())
            .storage(storage)
            .game(ruleset.clone())
            .threshold(100)
            .build()?;

        let about = format!("Forward pass of variant {}", ruleset.name());
        TaskBuilder::default()
            .executable(executable)
            .retriable(true)
            .about(about)
            .build()?
    };

    let mut scheduler = {
        let policy = CriticalPathPolicyBuilder::default().build()?;
        let runner = ThreadPoolRunnerBuilder::default().build()?;
        let logger = DashboardLoggerBuilder::default().build()?;

        let context = SchedulerContextBuilder::default()
            .policy(policy)
            .logger(logger)
            .runner(runner)
            .build()?;

        let state = SchedulerStateBuilder::default()
            .task(task)
            .build()?;

        SchedulerBuilder::default()
            .context(context)
            .state(state)
            .build()?
    };

    scheduler.run()
}

fn info(args: cli::InfoArgs) -> Result<()> {
    let data = match args.target {
        GameModule::ZeroBy => zero_by::Ruleset::info(),
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
