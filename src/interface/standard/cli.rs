//! # Command Line Module
//!
//! This module offers UNIX-like CLI tooling in order to facilitate scripting
//! and ergonomic use of GamesmanNova. This uses the
//! [clap](https://docs.rs/clap/latest/clap/) crate to provide standard
//! behavior, which is outlined in [this](https://clig.dev/) great guide.
//!
//! #### Authorship
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use clap::{Args, Parser, Subcommand};

use crate::interface::{
    GameAttribute, IOMode, InfoFormat, QueryFormat, Solution,
};
use crate::model::database::Identifier;
use crate::model::game::GameModule;

/* COMMAND LINE INTERFACE */

/// GamesmanNova is a project for searching sequential games. In addition to
/// being able to analyze games whose implementations are included distributed
/// along with the binary, the project also has database implementations that
/// can persist these analyses, which can later be queried.
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
    /// Solve some game through some specific method.
    Solve(SolveArgs),

    /// Run a query on an existing table in the system.
    Query(QueryArgs),

    /// Provides information about the system's offerings.
    Info(InfoArgs),
}

/* ARGUMENT AND OPTION DEFINITIONS */

/// Ensures a specific game variant's solution set exists. Default behavior:
///
/// - Uses the target's default variant (see `variant` argument).
/// - Attempts to read from a database file, computing and writing one only if
///   needed (see [`IOMode`] for specifics).
/// - Formats output aesthetically (see `output` argument).
/// - Finds an existing solution table to the game (see `solution` argument).
/// - Does not forward the game's starting state (see `from` argument).
/// - Prompts the user before executing any potentially destructive operations
///   such as overwriting a database file (see `yes` flag).
#[derive(Args)]
pub struct SolveArgs {
    /* REQUIRED ARGUMENTS */
    /// Target game name.
    pub target: GameModule,

    /* DEFAULTS PROVIDED */
    /// Solve a specific variant of target.
    #[arg(short, long)]
    pub variant: Option<String>,

    /// Specify what type of solution to compute.
    #[arg(short, long, default_value_t = Solution::Strong)]
    pub solution: Solution,

    /// Specify whether the solution should be fetched or re-generated.
    #[arg(short, long, default_value_t = IOMode::Constructive)]
    pub mode: IOMode,

    /// Compute solution starting after a state history read from STDIN.
    #[arg(short, long)]
    pub forward: bool,

    /// Skips prompts for confirming destructive operations.
    #[arg(short, long)]
    pub yes: bool,
}

/// Accepts a query string to be compiled and ran on a specific database table,
/// whose output table is printed to STDOUT. High-level behavior:
///
/// - `nova query` outputs the the global catalog table in `output` format.
/// - `nova query -t <T>` outputs the schema of table `<T>` in `output` format.
/// - `nova query -t <T> -q <Q>` outputs the result of the query `<Q>` on table
///   `<T>` in `output` format (but does not store it as a table).
#[derive(Args)]
pub struct QueryArgs {
    /* DEFAULTS PROVIDED */
    /// Numeric identifier for the table that the query should be run on.
    #[arg(short, long)]
    pub table: Option<Identifier>,

    /// Query specification string, conforming to ExQL syntax.
    #[arg(short, long)]
    pub query: Option<String>,

    /// Format in which to send output to STDOUT.
    #[arg(short, long, default_value_t = QueryFormat::CSV)]
    pub output: QueryFormat,
}

/// Provides information about games in the system. High-level behavior:
///
/// - `nova info <G>` outputs all known information about game `<G>` in
///   `output` format.
/// - `nova info <G> -a <A>` outputs the game `<G>`'s `<A>` attribute in
///   `output` format.
#[derive(Args)]
pub struct InfoArgs {
    /// Specify the game to provide information about.
    pub target: GameModule,

    /* DEFAULTS PROVIDED */
    /// Specify which of the game's attributes to provide information about.
    #[arg(short, long, value_delimiter = ',')]
    pub attributes: Vec<GameAttribute>,

    /// Format in which to send output to STDOUT.
    #[arg(short, long, default_value_t = InfoFormat::Legible)]
    pub output: InfoFormat,
}
