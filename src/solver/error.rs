//! # Solver Error Module
//!
//! This module defines possible errors that could happen during the execution
//! of a solving algorithm.

use std::{error::Error, fmt};

/* ERROR WRAPPER */

/// Wrapper for all solver-related errors that could happen during runtime. This
/// pertains specifically to the elements of the `crate::solver` module.
#[derive(Debug)]
pub enum SolverError {
    /// An error to indicate that the assumptions of a solving algorithm were
    /// detectably violated during execution.
    SolverViolation { name: String, hint: String },

    /// An error to indicate that there was an attempt to translate one measure
    /// into another incompatible measure. Provides hints about the input type,
    /// output type, and the reason behind the incompatibility.
    InvalidConversion {
        output_t: String,
        input_t: String,
        hint: String,
    },
}

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
