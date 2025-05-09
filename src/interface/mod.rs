//! # Interfaces Library
//!
//! This module provides all the available behavior used to interact with the
//! project in different ways, such as the command line.

use clap::ValueEnum;

use std::fmt;

/* UTILITY MODULES */

mod util;

/* INTERFACE IMPLEMENTATIONS */

pub mod cli;

/* DEFINITIONS */

/// Describes the format in which calls to the `info` CLI command to the binary
/// should print its output, which should be mostly human-readable.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum InfoFormat {
    /// Legible output intended for human eyes.
    Legible,

    /// Multi-platform compatible JSON format.
    Json,
}

/// Specifies a category of information kept about a game. Used for finding
/// specific information about game implementations through the `info` CLI
/// command. See [`crate::game::GameData`] for the provider data structure.
///
/// This level of granularity is supported for clients to automate
/// the creation of custom objects containing any choice of these without the
/// need to mangle this program's output.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum GameAttribute {
    /// The conventional name of the game, formatted to be unique.
    Name,

    /// The people involved in adding the game to the system.
    Authors,

    /// General introduction to the game's rules and setup.
    About,

    /// Explanation of how to encode a variant for the game.
    VariantProtocol,

    /// Regex pattern that all encodings of the game's variants must satisfy.
    VariantPattern,

    /// Default variant encoding the game uses when none is specified.
    VariantDefault,

    /// Explanation of how to encode a state for the game.
    StateProtocol,

    /// Regex pattern all encodings of the game's states must satisfy.
    StatePattern,

    /// The encoding of the game's default starting state.
    StateDefault,
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
    /// Use existing resources and compute whatever is missing.
    Constructive,

    /// Compute request from scratch, overwriting existing resources.
    Overwrite,

    /// Constructive, but does not persist any changes.
    Forgetful,
}

/* AUXILIARY IMPLEMENTATIONS */

impl fmt::Display for IOMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IOMode::Constructive => write!(f, "constructive"),
            IOMode::Overwrite => write!(f, "overwrite"),
            IOMode::Forgetful => write!(f, "forgetful"),
        }
    }
}

impl fmt::Display for InfoFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfoFormat::Legible => write!(f, "legible"),
            InfoFormat::Json => write!(f, "json"),
        }
    }
}
