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
    models::{PlayerCount, State},
};
use record::Record;

/* DBMS IMLPEMENTATIONS */

pub mod bpdb;
pub mod record;

/* TRAITS */

/// Database management system interface for storing game state to value
/// mappings.
pub trait Database<const N: PlayerCount> {
    /// Instantiate a new database.
    fn new(id: String, mode: IOMode) -> Self
    where
        Self: Sized;
    /// Create a new record.
    fn put(&mut self, state: State, record: Record<N>);
    /// Read a record. Returns `None` if record does not exist.
    fn get(&self, state: State) -> Option<Record<N>>;
    /// Delete a record.
    fn delete(&mut self, state: State);
}
