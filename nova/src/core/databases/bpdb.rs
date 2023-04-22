//! # Bit Perfect Database Module
//!
//! This module contains the implementation for a bit-perfect database.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

use std::collections::HashMap;
use std::sync::Mutex;

use super::Database;
use crate::core::{State, Value};

/// An implementation of a Bit-Perfect DBMS which exposes the option to force a
/// disk read, a write, or non-persistent behavior (at least beyond program
/// execution, as no guarantees are provided about disk usage limits during
/// execution).
pub struct BPDatabase {
    /// Used to identify the database file should the contents be persisted.
    id: String,
    read: bool,
    write: bool,
    mem: HashMap<State, Mutex<Value>>,
}

impl Database for BPDatabase {
    fn new(id: String, read: bool, write: bool) -> Self {
        if read && write {
            panic!("Cannot operate in read and write modes simultaneously.")
        }
        BPDatabase {
            id,
            read,
            write,
            mem: HashMap::new(),
        }
    }
 
    fn put(&mut self, state: State, value: Value) {
        if !self.read {
            if self.write {
                // Check if there is a logger thread, make one if not
                // Send update to logger thread
                // Once in a while, send signal to checkpoint thread
                todo!()
            }
            self.mem.insert(state, Mutex::new(value));
        }
    }

    fn get(&self, state: State) -> Option<Value> {
        if self.read {
            // Get checkpoint transmitter and send request for record with state
            // Wait for message including record using checkpoint transmitter
            // Return retrieved record
            todo!()
        } else {
            if let Some(mutex) = self.mem.get(&state) {
                let lock = mutex.lock().unwrap();
                Some(*lock)
            } else {
                None
            }
        }
    }

    fn delete(&mut self, state: State) {
        todo!()
    }
}
