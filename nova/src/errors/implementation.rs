//! # Implementation Errors Module
//!
//! This module defines the errors that can happen as a result of an
//! implementer's error, or of an incomplete implementation.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/13/2023 (maxfierro@berkeley.edu)

use crate::errors::NovaError;
use std::{error::Error, fmt};

/// Parent type for all implementer-sourced errors.
#[derive(Debug)]
pub enum ImplementationError {
    /// An error to indicate that a game's implementation returns an empty
    /// collection of solvers.
    NoSolversFoundError(String),
}

impl NovaError for ImplementationError {}
impl Error for ImplementationError {}

impl fmt::Display for ImplementationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoSolversFoundError(game_name) => {
                write!(f, "The game '{}' has no solvers implemented.", game_name)
            }
        }
    }
}
