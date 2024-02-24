//! # Solver Error Module
//!
//! This module defines possible errors that could happen during the execution
//! of a solving algorithm. Note that while this module is the main client of
//! database implementations, this is exclusive of database-related errors,
//! which can be found in `crate::database::error`.
//!
//! #### Authorship
//!
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use std::{error::Error, fmt};

/* ERROR WRAPPER */

/// Wrapper for all solver-related errors that could happen during runtime. This
/// pertains specifically to the elements of the `crate::solver` module.
#[derive(Debug)]
pub enum SolverError {}

impl Error for SolverError {}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            _ => todo!(),
        }
    }
}
