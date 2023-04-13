//! # User Errors Module
//!
//! This module defines the errors that can happen as a result of user input.
//! These errors are not for malformed input, but rather things that happen
//! as a result of the user not knowing the offerings of the program.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use crate::utils::most_similar;
use std::{error::Error, fmt};

use super::NovaError;

/// Parent type for all user-sourced errors.
#[derive(Debug)]
pub enum UserError {
    /// An error to indicate that a user input the name of a game which is not
    /// included as a target. Supports telling the user what they typed and
    /// a suggestion, presumably using a string distance calculator.
    GameNotFoundError(String),
    /// An error to indicate that a user input the name of a solver which is not
    /// imlpemented for a game. Supports telling the user what they typed and
    /// a suggestion, presumably using a string distance calculator.
    SolverNotFoundError(String, Vec<String>),
}

impl NovaError for UserError {}
impl Error for UserError {}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::GameNotFoundError(input) => {
                write!(
                    f,
                    "The game '{}' was not found among the offerings. Perhaps you meant '{}'?",
                    input,
                    most_similar(&input[0..], games::IMPLEMENTED_GAMES.to_vec())
                )
            }
            Self::SolverNotFoundError(name, list) => {
                write!(
                    f,
                    "The solver '{}' was not found among the offerings. Perhaps you meant '{}'?",
                    name,
                    most_similar(&name[0..], list.iter().map(|s| &s[0..]).collect())
                )
            }
        }
    }
}
