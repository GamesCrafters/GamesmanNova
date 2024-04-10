//! # Command Line Module
//!
//! This module offers UNIX-like CLI tooling in order to facilitate scripting
//! and ergonomic use of GamesmanNova. This uses the [clap](https://docs.rs/clap/latest/clap/)
//! crate to provide standard behavior, which is outlined in
//! [this](https://clig.dev/) great guide.
//!
//! #### Authorship
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use clap::{Args, Parser, Subcommand};

use crate::interface::{IOMode, OutputMode, SolutionMode};
use crate::util::GameModule;

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
    /// Compute solution starting after a file-provided state history.
    #[arg(short, long)]
    pub from: Option<String>,
    /// Specify what type of solution to compute.
    #[arg(short, long, default_value_t = SolutionMode::Strong)]
    pub solver: SolutionMode,
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
    /// Set output in a specific format.
    #[arg(short, long, default_value_t = OutputMode::Extra)]
    pub output: OutputMode,
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
    #[arg(short, long, default_value_t = OutputMode::Extra)]
    pub output: OutputMode,
}
