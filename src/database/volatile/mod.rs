//! # Volatile Database
//!
//! This module provides a trivial database implementation backed by a volatile
//! in-memory hashmap.
//!
//! #### Authorship
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use anyhow::Result;
use bitvec::{order::Msb0, slice::BitSlice, store::BitStore};

use std::collections::HashMap;

use crate::{
    database::{KVStore, Record, Schema, Tabular},
    model::State,
};


//TODO: efficient version: have a huge Vec chunk of memory, and HashMap just stores indexes in that memory chunk

pub struct Database {
    memory: HashMap<State, Vec<u8>>,
}

impl Database<'_> {
    pub fn initialize() -> Self {
        Self {
            memory: HashMap::new(),
        }
    }
}


impl KVStore for Database {
    fn put(&mut self, key: State, value: &[u8]) {
        let new = Vec::from(value).clone();
        self.memory.insert(key, new);
    }

    fn get(&self, key: State) -> Option<&[u8]> {
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



impl Tabular for Database<'_> {
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
