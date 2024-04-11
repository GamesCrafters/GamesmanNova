//! # Solver Error Module
//!
//! This module defines possible errors that could happen during the execution
//! of a solving algorithm. Note that while this module is the main client of
//! database implementations, this is exclusive of database-related errors,
//! which can be found in `crate::database::error`.
//!
//! #### Authorship
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use std::{error::Error, fmt};

/* ERROR WRAPPER */

/// Wrapper for all solver-related errors that could happen during runtime. This
/// pertains specifically to the elements of the `crate::solver` module.
#[derive(Debug)]
pub enum SolverError {
    /// An error to indicate that the limitations of a record implementation
    /// were exceeded during the execution of a solving algorithm by a consumer
    /// game implementation.
    RecordViolation { name: String, hint: String },

    /// An error to indicate that the assumptions of a solving algorithm were
    /// detectably violated during execution.
    SolverViolation { name: String, hint: String },

    /// An error to indicate that an error was made trying to use a utility value; for example,
    /// this can be used when a invalid bit sequence for a simple-sum utiliity is accessed
    UtilityViolation { name: String, hint: String },
}

impl Error for SolverError {}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecordViolation { name, hint } => {
                write!(
                    f,
                    "A limitation set by the record implementation '{}' was \
                    violated at runtime: {}",
                    name, hint,
                )
            },
            Self::SolverViolation { name, hint } => {
                write!(
                    f,
                    "An assumption set by the solver '{}' was violated at \
                    runtime: {}",
                    name, hint,
                )
            },
            Self::UtilityViolation { name, hint } => {
                write!(
                    f,
                    "An invalid utility for '{}' was violated at \
                    runtime: {}",
                    name, hint,
                )
            },

        }
    }
}
