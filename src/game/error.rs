//! # Game Error Module
//!
//! This module defines possible errors that could happen as a result of user
//! input or an incomplete game implementation.
//!
//! #### Authorship
//!
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use std::{error::Error, fmt};

/* ERROR WRAPPER */

/// Wrapper for all game-related errors that could happen during runtime. Note
/// that the elements of this enumeration are all related to the implementation
/// of interface elements in `crate::game::mod`.
#[derive(Debug)]
pub enum GameError {
    /// An error to indicate that a user attempted to solve a game variant
    /// which is valid, but has no solver available to solve it.
    SolverNotFound { input_game_name: &'static str },

    /// An error to indicate that the variant passed to the game with
    /// `game_name` was not in a format the game could parse. Includes a
    /// message from the game implementation on exactly what went wrong. Note
    /// that `game_name` should be a valid argument to the `--target`
    /// parameter in the CLI.
    VariantMalformed {
        game_name: &'static str,
        hint: String,
    },

    /// An error to indicate that the state string passed to the game with
    /// `game_name` was not in a format the game could parse. Includes a
    /// message from the game implementation on exactly what went wrong. Note
    /// that `game_name` should be a valid argument to the `--target`
    /// parameter in the CLI.
    StateMalformed {
        game_name: &'static str,
        hint: String,
    },

    /// An error to indicate that a sequence of states in string form would
    /// be impossible to reproduce in real play. Includes a message from the
    /// game implementation on exactly what went wrong. Note that `game_name`
    /// should be a valid argument to the `--target` parameter in the CLI.
    InvalidHistory {
        game_name: &'static str,
        hint: String,
    },

    /// An error to indicate that a game-building rule within the abstract
    /// extensive mock game implementation was violated during construction.
    /// Since this is intended as an internal feature, it is a very general
    /// error variant.
    MockViolation { hint: String },
}

impl Error for GameError {}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SolverNotFound { input_game_name } => {
                write!(
                    f,
                    "The variant you specified for the game {} has no solvers \
                    associated with it.",
                    input_game_name
                )
            },
            Self::VariantMalformed { game_name, hint } => {
                write!(
                    f,
                    "{}\n\n\tMore information on how the game expects you to \
                    format it can be found with 'nova info {} --output extra'.",
                    hint, game_name
                )
            },
            Self::StateMalformed { game_name, hint } => {
                write!(
                    f,
                    "{}\n\n\tMore information on how the game expects you to \
                    format it can be found with 'nova info {} --output extra'.",
                    hint, game_name
                )
            },
            Self::InvalidHistory { game_name, hint } => {
                write!(
                    f,
                    "{}\n\n\tMore information on the game's rules can be found \
                    with 'nova info {} --output extra'.",
                    hint, game_name
                )
            },
            Self::MockViolation { hint } => write!(f, "{}", hint),
        }
    }
}
