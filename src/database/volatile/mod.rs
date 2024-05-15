//! # Volatile Database
//!
//! This module provides a trivial database implementation backed by a volatile
//! in-memory hashmap.
//!
//! #### Authorship
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use anyhow::Result;
use bitvec::{order::Msb0, slice::BitSlice};

use crate::{
    database::{self, KVStore, Record, Schema, Tabular},
    model::database::{Identifier, Key},
};

/* DEFINITIONS */

pub struct Database {}

pub struct Table {}

/* IMPLEMENTATIONS */

impl Database {
    pub fn initialize() -> Self {
        Database {}
    }
}

impl Tabular<Table> for Database {
    fn create_table(
        &self,
        id: Identifier,
        schema: Schema,
    ) -> Result<&mut Table> {
        todo!()
    }

    fn select_table(&self, id: Identifier) -> Result<&mut Table> {
        todo!()
    }

    fn delete_table(&self, id: &mut Table) -> Result<()> {
        todo!()
    }
}

impl database::Table for Table {
    fn schema(&self) -> &Schema {
        todo!()
    }

    fn count(&self) -> u64 {
        todo!()
    }

    fn size(&self) -> u64 {
        todo!()
    }

    fn id(&self) -> Identifier {
        todo!()
    }
}

impl KVStore for Table {
    fn put<R: Record>(&mut self, key: &Key, value: &R) -> Result<()> {
        todo!()
    }

    fn get(&self, key: &Key) -> Option<&BitSlice<u8, Msb0>> {
        todo!()
    }

    fn delete(&mut self, key: &Key) {
        todo!()
    }
}
