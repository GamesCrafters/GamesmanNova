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

use crate::utils::most_similar;
use std::{error::Error, fmt};

/* UNIVERSAL WRAPPER */

/// Wrapper for all errors that could happen during runtime.
#[derive(Debug)]
pub enum NovaError
{
    /// An error to indicate that a user input the name of a solver which is
    /// not implemented for a game. Supports telling the user what they typed
    /// and a suggestion.
    SolverNotFound
    {
        input_solver_name: String,
        input_game_name: String,
        available_solvers: Vec<String>,
    },
    /// An error to indicate that the variant passed to the game with
    /// `game_name` was not in a format the game could parse. Includes a
    /// message from the game implementation on exactly what went wrong. Note
    /// that `game_name` should be a valid argument to the `--target`
    /// parameter in any command.
    VariantMalformed
    {
        game_name: String, hint: String
    },
}

impl Error for NovaError {}

impl fmt::Display for NovaError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self {
            Self::SolverNotFound {
                input_solver_name,
                input_game_name,
                available_solvers,
            } => {
                if available_solvers.is_empty() {
                    write!(
                        f,
                        "Sorry, the variant you specified for the game {} has \
                        no solvers associated with it.",
                        input_game_name
                    )
                } else {
                    write!(
                        f,
                        "The solver '{}' was not found among the offerings of \
                        the variant you specified for {}. Perhaps you meant \
                        '{}'?",
                        input_solver_name,
                        input_game_name,
                        most_similar(
                            input_solver_name,
                            available_solvers.iter().map(|s| &s[0..]).collect()
                        )
                    )
                }
            }
            Self::VariantMalformed { game_name, hint } => {
                write!(
                    f,
                    "The provided variant is malformed: {}\n\nMore information \
                    on how the game expects you to format it can be found with \
                    'nova info --target {} --output extra'.",
                    hint, game_name
                )
            }
        }
    }
}
