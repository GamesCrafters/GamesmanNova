//! # Volatile Database
//!
//! This module provides a trivial database implementation backed by a volatile
//! in-memory hashmap.
//!
//! #### Authorship
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)
//! - Implementation: Casey Stanford, 4/10/2024 (cqstanford@berkeley.edu)

use anyhow::Result;
use bitvec::{order::Msb0, prelude::*, slice::BitSlice, store::BitStore};

use std::collections::HashMap;

use crate::{
    database::{KVStore, Record, Schema, Tabular},
    model::State,
};

/// The Database struct holds an in-memory hashmap from States to BitVec values (derived from Records)
pub struct Database {
    memory: HashMap<State, BitVec<u8, Msb0>>,
}

impl Database {
    /// The database is initialized with an empty hashmap.
    pub fn initialize() -> Self {
        Self {
            memory: HashMap::new(),
        }
    }
}

/// The KVStore implementation includes three functions:
/// - put: takes in a State "key" and a Record "value", and stores the Record's raw value in its HashMap.
/// - get: takes in a State "key" and returns the raw value stored under that key as a BitSlice.
/// - del: takes in a State "key" and removes any value stored under that key.
impl KVStore for Database {
    fn put<R: Record>(&mut self, key: State, value: &R) {
        let new = BitVec::from(value.raw()).clone();
        self.memory.insert(key, new);
    }

    fn get(&self, key: State) -> Option<&BitSlice<u8, Msb0>> {
        let vecOpt = self.memory.get(&key);
        match vecOpt {
            None => None,
            Some(vect) => Some(&vect[..]),
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

    use super::*;

    /// A maximally simple Record implementation, consisting of only a BitVec value
    /// Created for test purposes
    pub struct Rec {
        value: BitVec<u8, Msb0>,
    }

    impl Rec {
        pub fn initialize(val: BitVec<u8, Msb0>) -> Self {
            Self { value: val.clone() }
        }
    }

    impl Record for Rec {
        fn raw(&self) -> &BitSlice<u8, Msb0> {
            return &self.value[..];
        }
    }

    /// This test
    /// - creates an example state test_state and Record test_rec
    /// - checks that that test_state is initially not mapped in the database
    /// - puts test_rec in the database, with test_state as its key
    /// - checks that test_state now maps to test_rec.
    #[test]
    fn put_data_and_get_it() {
        let mut db: Database = Database::initialize();
        let test_state: State = 7;
        assert!(db.get(test_state).is_none());
        let test_rec: Rec = Rec::initialize(BitVec::<u8, Msb0>::new());
        db.put(test_state, &test_rec);
        if let Some(result_rec) = db.get(test_state) {
            assert!(result_rec == test_rec.raw());
        } else {
            assert!(1 == 0);
        }
    }

    /// This test
    /// - creates an example state test_state and Record test_rec
    /// - puts test_rec in the database, with test_state as its key
    /// - deletes test_state and any associated Record
    /// - checks that test_state now, again, maps to nothing
    /// - puts test_rec BACK in the database, and confirms that test_state now maps to it once again.
    #[test]
    fn put_remove_and_put() {
        let mut db: Database = Database::initialize();
        let test_state: State = 7;
        let test_rec: Rec = Rec::initialize(BitVec::<u8, Msb0>::new());
        db.put(test_state, &test_rec);
        db.del(test_state);
        assert!(db.get(test_state).is_none());
        db.put(test_state, &test_rec);
        if let Some(result_rec) = db.get(test_state) {
            assert!(result_rec == test_rec.raw());
        } else {
            assert!(1 == 0);
        }
    }
}
