//! # Errors Module
//!
//! This module defines the errors that can happen during execution, only as
//! a result of a Nova-specific reason.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use crate::utils::most_similar;
use std::{error::Error, fmt};

/// Parent type for all user-sourced errors, such as malformed inputs.
#[derive(Debug)]
pub enum NovaError {
    /// An error to indicate that a user input the name of a game which is not
    /// included as a target. Supports telling the user what they typed and
    /// a suggestion, presumably using a string distance calculator.
    GameNotFoundError(String),
    /// An error to indicate that a user input the name of a solver which is not
    /// implemented for a game. Supports telling the user what they typed and
    /// a suggestion, presumably using a string distance calculator.
    SolverNotFoundError(String, Vec<String>),
}

impl Error for NovaError {}

impl fmt::Display for NovaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::GameNotFoundError(input) => {
                write!(
                    f,
                    "The game '{}' was not found among the offerings. Perhaps you meant '{}'?",
                    input,
                    most_similar(input, crate::games::LIST.to_vec())
                )
            }
            Self::SolverNotFoundError(name, list) => {
                write!(
                    f,
                    "The solver '{}' was not found among the offerings. Perhaps you meant '{}'?",
                    name,
                    most_similar(name, list.iter().map(|s| &s[0..]).collect())
                )
            }
        }
    }
}
