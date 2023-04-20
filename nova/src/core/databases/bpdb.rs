//! # Bit Perfect Database Module
//!
//! This module contains the implementation for a bit-perfect database.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

use super::Database;
use crate::core::{State, Value};

/// An implementation of a Bit-Perfect DBMS which exposes the option to force a
/// disk read, a write, or non-persistent behavior (at least beyond program
/// execution, as no guarantees are provided about disk usage limits during
/// execution).
pub struct BPDatabase {
    /// Used to identify the database file should the contents be persisted.
    id: String,
    /// Indicates that the database should try to fetch contents from a file,
    /// and fail if there is no such file.
    read: bool,
    /// Indicates that the database should persist its contents beyond runtime
    /// in a local file, or overwrite it should it already exist.
    write: bool,
}

impl Database for BPDatabase {
    fn new(id: String, read: bool, write: bool) -> Self {
        BPDatabase { id, read, write }
    }

    fn insert(&mut self, state: State, value: Option<Value>) {
        todo!()
    }

    fn get(&self, state: State) -> Option<&Value> {
        todo!()
    }

    fn delete(&mut self, state: State) {
        todo!()
    }
}
