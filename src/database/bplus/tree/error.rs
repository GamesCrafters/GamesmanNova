//! # Bplus Tree Errors [WIP]
//!
//! This file contains error codes that are
//! surfaced in the Bplus Tree API.

/* IMPORTS */

use std::fmt::{Display, Formatter, Result};

/* CONSTANTS */

/* DEFINITIONS */

#[derive(Debug)]
pub enum Error {
    TODO,
}

/* IMPLEMENTATIONS */

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        todo!()
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(_e: std::io::Error) -> Error {
        Error::UnexpectedError
    }
}

/* UNIT TESTING */

#[cfg(test)]
mod tests {
    use super::*;
}