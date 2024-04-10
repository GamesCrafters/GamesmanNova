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

pub struct Database<'a> {
    memory: HashMap<State, &'a [u8]>,
}

impl Database<'_> {
    pub fn initialize() -> Self {
        Self {
            memory: HashMap::new(),
        }
    }
}

impl<R: Record> KVStore<R> for Database<'_> {
    fn put(&mut self, key: State, value: &R) {
        todo!()
    }

    fn get(&self, key: State) -> Option<&BitSlice<u8, Msb0>> {
        todo!()
    }

    fn del(&self, key: State) {
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
