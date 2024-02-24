//! # Database Error Module
//!
//! This module defines possible errors that could happen while a database is
//! being executed. These errors should regard only the top-level module, not
//! any specific database implementation (in a sense, providing an abstraction
//! under which all database implementations' errors can be grouped into).
//!
//! #### Authorship
//!
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use std::{error::Error, fmt};

/* ERROR WRAPPER */

/// Wrapper for all database-interface-related errors that could happen during
/// runtime. This pertains specifically to the elements of the `crate::database`
/// module, and the interfaces it provides (not specific databases).
#[derive(Debug)]
pub enum DatabaseError {}

impl Error for DatabaseError {}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            _ => todo!(),
        }
    }
}
