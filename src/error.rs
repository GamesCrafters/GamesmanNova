//! # Error Implementations
//!
//! TODO

use std::error::Error;
use std::fmt;

/* ENUMERATIONS */

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

/// Wrapper for all solver-related errors that could happen during runtime. This
/// pertains specifically to the elements of the `crate::solver` module.
#[derive(Debug)]
pub enum SolverError {
    /// An error to indicate that the assumptions of a solving algorithm were
    /// detectably violated during execution.
    SolverViolation { name: String, hint: String },

    /// An error to indicate that limitations of a solver record were exceeded
    /// during the execution of a solving algorithm.
    RecordViolation { hint: String },

    /// An error to indicate that there was an attempt to translate one measure
    /// into another incompatible measure. Provides hints about the input type,
    /// output type, and the reason behind the incompatibility.
    InvalidConversion {
        output_t: String,
        input_t: String,
        hint: String,
    },
}

/* IMPL EXTERNAL TRAIT */

impl Error for GameError {}

impl Error for SolverError {}

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

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SolverViolation { name, hint } => {
                write!(
                    f,
                    "An assumption set by the solver '{name}' was violated at \
                    runtime: {hint}",
                )
            },
            Self::RecordViolation { hint } => {
                write!(
                    f,
                    "A limitation in a solver record buffer was violated at \
                    runtime: {hint}",
                )
            },
            Self::InvalidConversion {
                output_t,
                input_t,
                hint,
            } => {
                write!(
                    f,
                    "There was an attempt to translate a value of type \
                    '{input_t}' into a value of type '{output_t}': {hint}",
                )
            },
        }
    }
}
