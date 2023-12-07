//! # Bit Perfect Database Module
//!
//! This module contains the implementation for a bit-perfect database.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

use std::collections::HashMap;
use std::sync::Mutex;

use super::{Database, Record};
use crate::{
    interfaces::terminal::cli::IOMode,
    models::{PlayerCount, State},
};

/// An implementation of a Bit-Perfect DBMS which exposes the option to force a
/// disk read, a write, or non-persistent behavior (at least beyond program
/// execution, as no guarantees are provided about disk usage limits during
/// execution).
pub struct BPDatabase<const N: PlayerCount> {
    /// Used to identify the database file should the contents be persisted.
    id: String,
    mode: IOMode,
    mem: HashMap<State, Mutex<Record<N>>>,
}

impl<const N: PlayerCount> Database<N> for BPDatabase<N> {
    fn new(id: String, mode: IOMode) -> Self {
        BPDatabase {
            id,
            mode,
            mem: HashMap::new(),
        }
    }

    fn put(&mut self, state: State, record: Record<N>) {
        self.mem
            .insert(state, Mutex::new(record));
    }

    fn get(&self, state: State) -> Option<Record<N>> {
        if let Some(mutex) = self.mem.get(&state) {
            let lock = mutex.lock().unwrap();
            Some(*lock)
        } else {
            None
        }
    }

    fn delete(&mut self, state: State) {
        todo!()
    }
}
