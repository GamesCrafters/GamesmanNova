//! # Errors Module
//!
//! This module defines the errors that can happen during execution, only as
//! a result of a Nova-specific reason. Some examples of this are:
//!
//! - Malformed game variant strings provided to game implementations.
//! - A game or solver not being found among the offerings.
//! - Reading an incorrectly serialized solution set database.
//!
//! #### Authorship
//!
//! - Max Fierro, 11/2/2023 (maxfierro@berkeley.edu)

use std::{error::Error, fmt};

/* UNIVERSAL WRAPPER */

/// Wrapper for all errors that could happen during runtime.
#[derive(Debug)]
pub enum NovaError {
    /// An error to indicate that a user attempted to solve a game variant
    /// which is valid, but has no solver available to solve it.
    SolverNotFound { input_game_name: String },
    /// An error to indicate that the variant passed to the game with
    /// `game_name` was not in a format the game could parse. Includes a
    /// message from the game implementation on exactly what went wrong. Note
    /// that `game_name` should be a valid argument to the `--target`
    /// parameter in any command.
    VariantMalformed { game_name: String, hint: String },
    /// An error to indicate that the state string passed to the game with
    /// `game_name` was not in a format the game could parse. Includes a
    /// message from the game implementation on exactly what went wrong. Note
    /// that `game_name` should be a valid argument to the `--target`
    /// parameter in any command.
    StateMalformed { game_name: String, hint: String },
}

impl Error for NovaError {}

impl fmt::Display for NovaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SolverNotFound { input_game_name } => {
                write!(
                    f,
                    "Sorry, the variant you specified for the game {} has no \
                    solvers associated with it.",
                    input_game_name
                )
            },
            Self::VariantMalformed { game_name, hint } => {
                write!(
                    f,
                    "The provided variant is malformed: {}\n\nMore information \
                    on how the game expects you to format it can be found with \
                    'nova info --target {} --output extra'.",
                    hint, game_name
                )
            },
            Self::StateMalformed { game_name, hint } => {
                write!(
                    f,
                    "The provided state is malformed: {}\n\nMore information \
                    on how the game expects you to format it can be found with \
                    'nova info --target {} --output extra'.",
                    hint, game_name
                )
            },
        }
    }
}
