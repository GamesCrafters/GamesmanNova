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
        todo!()
    }

    fn get(&self, key: State) -> Option<&[u8]> {
        todo!()
    }

    fn del(&mut self, key: State) {
        todo!()
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
