//! # Interfaces Library
//!
//! This module provides all the available behavior used to interact with the
//! project in different ways, such as the command line.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use clap::ValueEnum;

use std::fmt;

/* UTILITY MODULES */

mod error;
mod util;

/* INTERFACE IMPLEMENTATIONS */

pub mod terminal;

/* DEFINITIONS */

/// Allows calls to return output in different formats for different purposes,
/// such as web API calls, scripting, or human-readable output.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputMode {
    /// Extra content or formatting where appropriate.
    Extra,

    /// Multi-platform compatible JSON format.
    Json,

    /// Output nothing (side-effects only).
    None,
}

/// Specifies how exhaustive a solving algorithm should be when computing a
/// solution set. Different algorithms will be used for computing a strong
/// solution (e.g., minimax) versus a weak solution (e.g., alpha-beta pruning).
///
/// Note that specifying a weak solution on a specific game variant does not
/// guarantee that the solving algorithm will traverse less states than the
/// strong alternative. The relative convenience of a weak solution relies on
/// the structure of the underlying game.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SolutionMode {
    /// Minimally prove an optimal strategy beginning from a starting state.
    Weak,

    /// Provide a strategy for all game states reachable from starting state.
    Strong,
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
///
/// 2. If an insufficient database file exists and is not corrupted, the
/// existing information about the solution to the underlying game should be
/// used to produce the remainder of the request.
///
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

/* AUXILIARY IMPLEMENTATIONS */

impl fmt::Display for IOMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IOMode::Find => write!(f, "find"),
            IOMode::Write => write!(f, "write"),
        }
    }
}

impl fmt::Display for SolutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolutionMode::Weak => write!(f, "weak"),
            SolutionMode::Strong => write!(f, "strong"),
        }
    }
}

impl fmt::Display for OutputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputMode::Json => write!(f, "json"),
            OutputMode::Extra => write!(f, "extra"),
            OutputMode::None => write!(f, "none"),
        }
    }
}
