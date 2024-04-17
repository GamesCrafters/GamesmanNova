//! # Volatile Database
//!
//! This module provides a trivial database implementation backed by a volatile
//! in-memory hashmap.
//!
//! #### Authorship
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)
//! - Implementation: Casey Stanford, 4/10/2024 (cqstanford@berkeley.edu)

use anyhow::Result;
use bitvec::{prelude::*, order::Msb0, slice::BitSlice, store::BitStore};

use std::collections::HashMap;

use crate::{
    database::{KVStore, Record, Schema, Tabular},
    model::State,
};


//TODO: efficient version: have a huge Vec chunk of memory, and HashMap just stores indexes in that memory chunk

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
    fn put<R:Record>(&mut self, key: State, value: &R) {
        let new = BitVec::from(value.raw()).clone();
        self.memory.insert(key, new);
    }

    fn get(&self, key: State) -> Option<&BitSlice<u8, Msb0>> {
        let vecOpt = self.memory.get(&key);
        match vecOpt {
            None => None,
            Some(vect) => Some(&vect[..])
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

    pub struct Rec {
        value: BitVec<u8, Msb0>,
    }

    impl Rec {
        pub fn initialize(val: BitVec<u8, Msb0>) -> Self {
            Self {
                value: val.clone(),
            }
        }
    }

    impl Record for Rec {
        fn raw(&self) -> &BitSlice<u8, Msb0> {
            return &self.value[..];
        }
    }

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
}
