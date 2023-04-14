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

use clap::{Args, Parser, Subcommand, ValueEnum};

/* COMMAND LINE INTERFACE */

/// GamesmanNova is a project for solving finite-state deterministic abstract
/// strategy games. In addition to solving one of the implemented games, there
/// are also domain-specific analyzers and databases available to provide
/// insight about games and to store their full solutions efficiently.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /* REQUIRED COMMANDS */
    /// Available subcommands for the main 'nova' command.
    #[command(subcommand)]
    pub command: Commands,

    /* DEFAULTS PROVIDED */
    /// Send no output to STDOUT during execution.
    #[arg(short, long, group = "out")]
    pub quiet: bool,
    /// Skips prompts for confirming destructive operations.
    #[arg(short, long)]
    pub yes: bool,
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
    /// Provide available functionality.
    List(ListArgs),
}

/* ARGUMENT AND OPTION DEFINITIONS */

/// Specifies the way in which the TUI is initialized. By default, this will
/// open a main menu which allows the user to choose which game to play among
/// the list of available games, in addition to other miscellaneous offerings.
#[derive(Args)]
pub struct TuiArgs {
    /* DEFAULTS PROVIDED */
    /// Game to display (optional).
    #[arg(short, long)]
    pub target: Option<String>,
    /// Enter TUI in debug mode.
    #[arg(short, long)]
    pub debug: bool,
}

/// Specifies the way in which a solve for a game happens and returns the
/// value and remoteness of the `target` game's initial position. Default
/// behavior:
/// * Does not attempt to read from a pre-generated database (see `read` flag).
/// * Does not attempt to generate a database (see `write` flag).
/// * Formats output aesthetically (see `output` argument).
/// * Uses the game's default solver to create state graph (see `solver`
/// argument).
#[derive(Args)]
pub struct SolveArgs {
    /* REQUIRED ARGUMENTS */
    /// Target game name.
    #[arg(short, long)]
    pub target: String,

    /* DEFAULTS PROVIDED */
    /// Solve a specific variant of target.
    pub variant: Option<String>,
    /// Set output in a specific format.
    #[arg(short, long)]
    pub output: Option<Output>,
    /// Attempt to use a specific solver to solve the game.
    #[arg(short, long)]
    pub solver: Option<String>,
    /// Write or overwrite solved game state graph to local database.
    #[arg(short, long, group = "read-write")]
    pub write: bool,
    /// Read solved game state graph from local database.
    #[arg(short, long, group = "read-write")]
    pub read: bool,
}

/// Specifies the way in which a game's analysis happens. Uses the provided
/// `analyzer` to analyze the `target` game. Default behavior:
/// * Attempts to read from the game's solution database, and use it to make
/// the analysis.
/// * If there is no database, run the default solver for the game, and return
/// the analysis without writing a database.
/// * Should writing to disk be necessary to perform the solve, the database
/// file is deleted once the `analyzer` is finished.
#[derive(Args)]
pub struct AnalyzeArgs {
    /* REQUIRED ARGUMENTS */
    /// Analyzer to use.
    pub analyzer: String,
    /// Target game name.
    #[arg(short, long)]
    pub target: String,

    /* DEFAULTS PROVIDED */
    /// Analyze a specific variant of target.
    pub variant: Option<String>,
    /// Only perform the analysis if there is already a pre-existing database.
    #[arg(short, long)]
    pub read: bool,
    /// Set output in a specific format.
    #[arg(short, long)]
    pub output: Option<Output>,
}

/// Provides a list of implemented functionality that can be used as arguments
/// for other commands. Default behavior:
/// * Provides a list of implemented games (which are valid `--target`s)
#[derive(Args)]
pub struct ListArgs {
    /* DEFAULTS PROVIDED */
    /// Set output in a specific format.
    #[arg(short, long)]
    pub output: Option<Output>,
}

/* DEFINITIONS */

/// Allows calls to return output in different formats for different purposes,
/// such as web API calls, scripting, or simple human-readable output.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Output {
    /// Readable output.
    Formatted,
    /// JSON output.
    Json,
}
