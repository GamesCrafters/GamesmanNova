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

/* RESOURCE NOT FOUND ERRORS */

/// Indicates that a Nova resource which the user specified was not found or
/// does not exist. This can be a game, solver, DBMS, etc.
#[derive(Debug)]
pub enum NotFoundError
{
    /// An error to indicate that a user input the name of a solver which is
    /// not implemented for a game. Supports telling the user what they
    /// typed and a suggestion.
    Solver
    {
        solver_name: String,
        game_name: String,
        available_solvers: Vec<String>,
    },
}

impl Error for NotFoundError {}

impl fmt::Display for NotFoundError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self {
            Self::Solver {
                solver_name,
                game_name,
                available_solvers,
            } => {
                if available_solvers.is_empty() {
                    write!(
                        f,
                        "Sorry, the variant you specified for the game {} has \
                        no solvers associated with it.",
                        game_name
                    )
                } else {
                    write!(
                        f,
                        "The solver '{}' was not found among the offerings of \
                        {}. Perhaps you meant '{}'?",
                        solver_name,
                        game_name,
                        most_similar(
                            solver_name,
                            available_solvers.iter().map(|s| &s[0..]).collect()
                        )
                    )
                }
            }
        }
    }
}

/* VARIANT ERRORS */

/// Indicates that there is something wrong with the variant passed into a game
/// implementation, and provides information on how to fix the issue.
#[derive(Debug)]
pub enum VariantError
{
    /// An error to indicate that the variant passed to the game with
    /// `game_name` was not in a format the game could parse. Includes a
    /// message from the game implementation on exactly what went wrong. Note
    /// that `game_name` should be a valid argument to the `--target`
    /// parameter in any command.
    Malformed
    {
        game_name: String, message: String
    },
}

impl Error for VariantError {}

impl fmt::Display for VariantError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self {
            Self::Malformed { game_name, message } => {
                write!(
                    f,
                    "The provided variant is malformed: {}\n\nMore information \
                    on how the game expects you to format it can be found with \
                    'nova info --target {} --output extra'.",
                    message, game_name
                )
            }
        }
    }
}
