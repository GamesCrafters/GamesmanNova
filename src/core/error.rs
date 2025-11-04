//! # Error Implementations
//!
//! TODO

use std::error::Error;
use std::fmt;

use crate::types::error::GameError;
use crate::types::error::SolverError;

/* GAME ERRORS */

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

/* SOLVER ERRORS */

impl Error for SolverError {}
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
