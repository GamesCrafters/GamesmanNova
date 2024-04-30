//! # Volatile Database
//!
//! This module provides a trivial database implementation backed by a volatile
//! in-memory hashmap.
//!
//! #### Authorship
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)
//! - Casey Stanford, 4/10/2024 (cqstanford@berkeley.edu)

use anyhow::Result;
use bitvec::{order::Msb0, prelude::*, slice::BitSlice, store::BitStore};

use std::collections::HashMap;

use crate::{
    database::{KVStore, Record, Schema, Tabular},
    model::State,
    solver::record::rem,
};

/// [`KVStore`] implementation backed by an in-memory [`HashMap`].
/// Constrained by the space available in memory, growing at O(n) with the number of entries.
pub struct Database {
    memory: HashMap<State, BitVec<u8, Msb0>>,
}

impl Database {
    pub fn initialize() -> Self {
        Self {
            memory: HashMap::new(),
        }
    }
}

impl KVStore for Database {
    fn put<R: Record>(&mut self, key: State, value: &R) {
        let new = BitVec::from(value.raw()).clone();
        self.memory.insert(key, new);
    }

    fn get(&self, key: State) -> Option<&BitSlice<u8, Msb0>> {
        if let Some(vect) = self.memory.get(&key) {
            return Some(&vect[..]);
        } else {
            return None;
        }
    }

    fn del(&mut self, key: State) {
        self.memory.remove(&key);
    }
}

impl Tabular for Database {
    fn create_table(&self, id: &str, schema: Schema) -> Result<()> {
        todo!()
    }

    fn select_table(&self, id: &str) -> Result<()> {
        todo!()
    }

    fn delete_table(&self, id: &str) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::database::volatile::tests::rem::RecordBuffer;

    use super::*;

    /// This test:
    /// - Creates an example state test_state and Record test_rec.
    /// - Checks that that test_state is initially not mapped in the database.
    /// - Puts test_rec in the database, with test_state as its key.
    /// - Checks that test_state now maps to test_rec.
    #[test]
    fn put_data_and_get_it() {
        let mut db: Database = Database::initialize();
        let test_state: State = 7;
        assert!(db.get(test_state).is_none());
        let test_rec: RecordBuffer = RecordBuffer::new().unwrap();
        db.put(test_state, &test_rec);
        if let Some(result_rec) = db.get(test_state) {
            assert_eq!(result_rec, test_rec.raw());
        } else {
            assert_eq!(1, 0);
        }
    }

    /// This test
    /// - Creates an example state test_state and Record test_rec.
    /// - Puts test_rec in the database, with test_state as its key.
    /// - Deletes test_state and any associated Record.
    /// - Checks that test_state now, again, maps to nothing.
    /// - Puts test_rec BACK in the database, and confirms that test_state now maps to it once again.
    #[test]
    fn put_remove_and_put() {
        let mut db: Database = Database::initialize();
        let test_state: State = 7;
        let test_rec: RecordBuffer = RecordBuffer::new().unwrap();
        db.put(test_state, &test_rec);
        db.del(test_state);
        assert!(db.get(test_state).is_none());
        db.put(test_state, &test_rec);
        if let Some(result_rec) = db.get(test_state) {
            assert_eq!(result_rec, test_rec.raw());
        } else {
            assert_eq!(1, 0);
        }
    }
}
