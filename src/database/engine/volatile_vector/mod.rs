//! # Volatile Database
//!
//! This module provides a trivial database implementation backed by a volatile
//! in-memory hashmap.
//!
//! #### Authorship
//!
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use anyhow::Result;

use std::collections::HashMap;

use crate::{
    database::{object::schema::Schema, KVStore, Tabular},
    model::State,
};

pub struct Database {
    mapping: HashMap<State, u32>,
    memory: Vec<u8>,

}

impl Database<'_> {
    pub fn initialize() -> Self {
        Self {
            mapping: HashMap::new(),
            memory: Vec::new(),
        }
    }
}

impl KVStore for Database {
    fn put(&mut self, key: State, value: &[u8]) {
        let new = (value[0]).clone();
        self.mapping.insert(key, self.memory.len());
        self.memory.push(new);
    }

    fn get(&self, key: State) -> Option<&[u8]> {
        let indexOpt = self.mapping.get(&key);
        match indexOpt {
            None => None,
            Some(index) => Some(self.memory[*index])
        }
        //Doesn't work yet
    }

    fn del(&mut self, key: State) {
        self.mapping.remove(&key);
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
