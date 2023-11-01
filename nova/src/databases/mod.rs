//! # Databases Module
//!
//! This module contains the I/O subroutines to store the results of the
//! solving algorithms persistently.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

use crate::{
    interfaces::terminal::cli::IOMode,
    models::{Record, State},
};

/* DBMS IMLPEMENTATIONS */

pub mod bpdb;

/* TRAITS */

/// Database management system interface for storing game state to value
/// mappings.
pub trait Database
{
    /// Instantiate a new database.
    fn new(id: String, mode: Option<IOMode>) -> Self
    where
        Self: Sized;
    /// Create a new record.
    fn put(&mut self, state: State, value: Record);
    /// Read a record. Returns `None` if record does not exist.
    fn get(&self, state: State) -> Option<Record>;
    /// Delete a record.
    fn delete(&mut self, state: State);
}
