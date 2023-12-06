//! # Command Line Module
//!
//! This module offers UNIX-like CLI tooling in order to facilitate scripting
//! and ergonomic use of GamesmanNova. This uses the [clap](https://docs.rs/clap/latest/clap/)
//! crate to provide standard behavior, which is outlined in
//! [this](https://clig.dev/) great guide.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::interfaces::GameModule;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt;

/* COMMAND LINE INTERFACE */

/// GamesmanNova is a project for solving finite-state, deterministic, abstract
/// strategy games. In addition to being able to solve implemented games, Nova
/// provides analyzers and databases to generate insights about games and to
/// persist their full solutions efficiently.
#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct Cli {
    /* REQUIRED COMMANDS */
    /// Available subcommands for the main 'nova' command.
    #[command(subcommand)]
    pub command: Commands,

    /* DEFAULTS PROVIDED */
    /// Send no output to STDOUT during successful execution.
    #[arg(short, long, group = "output")]
    pub quiet: bool,
}

/// Subcommand choices, specified as `nova <subcommand>`.
#[derive(Subcommand)]
pub enum Commands {
    /// Start the terminal user interface.
    Tui(TuiArgs),

    /// Solve a game from the start position.
    Solve(SolveArgs),

    /// Analyze a game's state graph.
    Analyze(AnalyzeArgs),

    /// Provide information about offerings.
    Info(InfoArgs),
}

/* ARGUMENT AND OPTION DEFINITIONS */

/// Specifies the way in which the TUI is initialized. By default, this will
/// open a main menu which allows the user to choose which game to play among
/// the list of available games, in addition to other miscellaneous offerings,
/// and prompt the user for confirmation before executing any potentially
/// destructive operations.
#[derive(Args)]
pub struct TuiArgs {
    /* DEFAULTS PROVIDED */
    /// Game to display (optional).
    #[arg(short, long)]
    pub target: Option<GameModule>,
    /// Enter TUI in debug mode.
    #[arg(short, long)]
    pub debug: bool,
    /// Skips prompts for confirming destructive operations.
    #[arg(short, long)]
    pub yes: bool,
}

/// Ensures a specific game variant's solution set exists. Default behavior:
///
/// - Uses the target's default variant (see `variant` argument).
/// - Attempts to read from a database file, computing and writing one only if
/// needed (see `cli::IOMode` for specifics).
/// - Formats output aesthetically (see `output` argument).
/// - Uses the game's default solver to create state graph (see `solver`
/// argument).
/// - Prompts the user before executing any potentially destructive operations
/// such as overwriting a database file (see `yes` flag).
#[derive(Args)]
pub struct SolveArgs {
    /* REQUIRED ARGUMENTS */
    /// Target game name.
    pub target: GameModule,

    /* DEFAULTS PROVIDED */
    /// Solve a specific variant of target.
    #[arg(short, long)]
    pub variant: Option<String>,
    /// Set output in a specific format.
    #[arg(short, long)]
    pub output: Option<OutputFormat>,
    /// Compute a weak solution from an encoded state.
    #[arg(short, long)]
    pub from: Option<String>,
    /// Specify whether the solution should be fetched or generated.
    #[arg(short, long, default_value_t = IOMode::Find)]
    pub mode: IOMode,
    /// Skips prompts for confirming destructive operations.
    #[arg(short, long)]
    pub yes: bool,
}

/// Specifies the way in which a game's analysis happens. Uses the provided
/// `analyzer` to analyze the `target` game. This uses the same logic on finding
/// or generating missing data as the solving routine; see `cli::IOMode` for
/// specifics.
#[derive(Args)]
pub struct AnalyzeArgs {
    /* REQUIRED ARGUMENTS */
    /// Target game name.
    pub target: GameModule,

    /* DEFAULTS PROVIDED */
    /// Analyzer module to use.
    #[arg(short, long)]
    pub analyzer: Option<String>,
    /// Analyze a specific variant of target.
    #[arg(short, long)]
    pub variant: Option<String>,
    /// Specify whether the solution should be fetched or generated.
    #[arg(short, long, default_value_t = IOMode::Find)]
    pub mode: IOMode,
    /// Set output in a specific format.
    #[arg(short, long)]
    pub output: Option<OutputFormat>,
    /// Skips prompts for confirming destructive operations.
    #[arg(short, long)]
    pub yes: bool,
}

/// Provides information about available games (or about their specifications,
/// if provided a `--target` argument). Default behavior:
///
/// - Provides a list of implemented games (which are valid `--target`s).
/// - Provides output unformatted.
#[derive(Args)]
pub struct InfoArgs {
    /* REQUIRED ARGUMENTS */
    /// Specify game for which to provide information about.
    pub target: GameModule,

    /* DEFAULTS PROVIDED */
    /// Set output in a specific format.
    #[arg(short, long)]
    pub output: Option<OutputFormat>,
}

/* DEFINITIONS */

/// Allows calls to return output in different formats for different purposes,
/// such as web API calls, scripting, or simple human-readable output.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputFormat {
    /// Extra content or formatting where appropriate.
    Extra,

    /// JSON format.
    Json,

    /// Output nothing (side-effects only).
    None,
}

/// Specifies a mode of operation for solving algorithms in regard to database
/// usage and solution set persistence. There are a few cases to consider about
/// database files every time a command is received:
///
/// - It exists and is complete (it is a strong solution).
/// - It exists but is incomplete (it is a weak solution).
/// - It exists but is corrupted.
/// - It does not exist.
///
/// For each of these cases, we can have a user try to compute a strong, weak,
/// or stochastic solution (under different equilibrium concepts) depending on
/// characteristics about the game. Some of these solution concepts will be
/// compatible with each other (e.g., a strong solution is a superset of a weak
/// one, and some stochastic equilibria are _stronger_ than others). We can use
/// this compatibility to eschew unnecessary work by considering the following
/// scenarios:
///
/// 1. If an existing database file exists, is not corrupted, and sufficient, it
/// will be used to serve a request. For example, if there is an existing strong
/// solution on a game and a command is issued to compute a weak solution for
/// it, then nothing should be done.
/// 2. If an insufficient database file exists and is not corrupted, the
/// existing information about the solution to the underlying game should be
/// used to produce the remainder of the request.
/// 3. Finally, if a database file does not exist or is corrupted (beyond any
/// possibility of repair by a database recovery mechanism), then it will be
/// computed again up to the number of states associated with the request.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum IOMode {
    /// Attempt to find an existing solution set to use or expand upon.
    Find,

    /// Overwrite any existing solution set that could contain the request.
    Write,
}

/* UTILITY IMPLEMENTATIONS */

impl fmt::Display for IOMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IOMode::Find => write!(f, "find"),
            IOMode::Write => write!(f, "write"),
        }
    }
}
