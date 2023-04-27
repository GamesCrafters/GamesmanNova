//! # Databases Module
//!
//! This module contains the I/O subroutines to store the results of the
//! solving algorithms persistently.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

use std::path::Path;

use crate::core::{State, Value};

/* DBMS IMLPEMENTATIONS */

pub mod streamdb;

/* TRAITS */

/// Database management system interface for storing game state to value
/// mappings.
pub trait Database {
    /// Instantiate a new database. If `read` is true, it will attempt to read
    /// an existing database using the information in `id`. If write is true,
    /// it will attempt to overwrite or write a new database with `id`. If
    /// neither are true, it does no disk I/O.
    fn new(name: String, mode: DatabaseMode) -> Self
    where
        Self: Sized;
    /// Create a new record.
    fn put(&mut self, state: State, value: Value);
    /// Read a record. Returns `None` if record does not exist.
    fn get(&self, state: State) -> Option<Value>;
    /// Delete a record.
    fn delete(&mut self, state: State);
}

/// Indicates the mode a database should operate in.
pub enum DatabaseMode {
    /// Makes no attempt at persisting the information passed in.
    Virtual,
    /// Only sources data from a file with the same name as the database.
    ReadOnly,
    /// Persists the state information provided into a file with the same name.
    Default,
}
