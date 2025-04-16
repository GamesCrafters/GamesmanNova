//! # Game Error Module
//!
//! This module defines possible errors that could happen as a result of user
//! input or an incomplete game implementation.

use std::{error::Error, fmt};

/* ERROR WRAPPER */

/// Wrapper for all game-related errors that could happen during runtime. Note
/// that the elements of this enumeration are all related to the implementation
/// of interface elements in `crate::game::mod`.
#[derive(Debug)]
pub enum GameError {
    /// An error to indicate that the variant passed to the game with the name
    /// `game` was not in a format the game could parse. Includes a message
    /// from the game implementation on exactly what went wrong. Note: `game`
    /// should be a valid argument to the `--target` parameter in the CLI.
    VariantMalformed { game: &'static str, hint: String },

    /// An error to indicate that the state string passed to the game with the
    /// name `game` was not in a format the game could parse. Includes a message
    /// from the game implementation on exactly what went wrong. Note: `game`
    /// should be a valid argument to the `--target` parameter in the CLI.    
    StateMalformed { game: &'static str, hint: String },

    /// An error to indicate that a sequence of states in string form would
    /// be impossible to reproduce in real play. Includes a message from the
    /// game implementation on exactly what went wrong. Note: `target_name`
    /// should be a valid argument to the `--target` parameter in the CLI.
    InvalidHistory { game: &'static str, hint: String },
}

impl Error for GameError {}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::VariantMalformed { game, hint } => {
                write!(
                    f,
                    "{hint}\n\nMore information on how the game expects you to \
                    format variant encodings can be found with 'nova info \
                    {game}'.",
                )
            },
            Self::StateMalformed { game, hint } => {
                write!(
                    f,
                    "{hint}\n\nMore information on how the game expects you to \
                    format state encodings can be found with 'nova info \
                    {game}'.",
                )
            },
            Self::InvalidHistory { game, hint } => {
                write!(
                    f,
                    "{hint}\n\nMore information on the game's rules can be \
                    found with 'nova info {game}'.",
                )
            },
        }
    }
}
