//! # Databases Module
//!
//! This module contains the I/O subroutines to store the results of the
//! solving algorithms persistently.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

use crate::core::{State, Value};

/* DBMS IMLPEMENTATIONS */

pub mod bpdb;

/* TRAITS */

/// Database management system interface for storing game state to value
/// mappings.
pub trait Database {
    /// Instantiate a new database. If `read` is true, it will attempt to read
    /// an existing database using the information in `id`. If write is true,
    /// it will attempt to overwrite or write a new database with `id`. If
    /// neither are true, it does no disk I/O.
    fn new(id: String, read: bool, write: bool) -> Self
    where
        Self: Sized;
    /// Create a new record.
    fn insert(&mut self, state: State, value: Option<Value>);
    /// Read a record. Returns `None` if record does not exist.
    fn get(&self, state: State) -> Option<&Value>;
    /// Delete a record.
    fn delete(&mut self, state: State);
}
