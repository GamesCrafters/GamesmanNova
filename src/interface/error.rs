//! # Interface Error Module
//!
//! This module defines possible errors that could happen as a result of program
//! interfaces, both from external (e.g., being provided bad file descriptors)
//! and internal (e.g., failing to spawn a child process) sources.
//!
//! #### Authorship
//!
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use std::{error::Error, fmt};

/* ERROR WRAPPER */

/// Wrapper for all interface-related errors that could happen during runtime.
/// This pertains specifically to the elements of the `crate::interface` module.
#[derive(Debug)]
pub enum InterfaceError {}

impl Error for InterfaceError {}

impl fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => todo!(),
        }
    }
}
