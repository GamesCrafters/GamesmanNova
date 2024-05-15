//! # Vector Database
//!
//! This module contains a very simple implementation of a persistent key-value
//! store. It works by indexing into an allocated vector through keys, always
//! making sure that it is large enough to house the record with the highest
//! key. This means that its top capacity is the amount of memory that can be
//! allocated by the operating system, without considering the usage of virtual
//! memory.
//!
//! For persistence, a file is created containing a bit-accurate representation
//! of the in-memory vector. Table logic is handled by switching which of these
//! files is currently being targeted, with the understanding that the contents
//! of memory may be materialized on arbitrary operations.
//!
//! #### Authorship
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

use anyhow::Result;

use std::path::Path;

use crate::{
    database::{self, KVStore, Persistent, Record, Schema, Tabular},
    model::database::{Identifier, Key, Value},
};

/* DEFINITIONS */

pub struct Database {}

pub struct Table {}

/* IMPLEMENTATIONS */

impl Persistent<Table> for Database {
    fn from(path: &Path) -> Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn bind(&self, path: &Path) -> Result<()> {
        todo!()
    }

    fn flush(&self, table: &mut Table) -> Result<()> {
        todo!()
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        todo!()
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
    fn put<R>(&mut self, key: &Key, value: &R) -> Result<()>
    where
        R: Record,
    {
        todo!()
    }

    fn get(&self, key: &Key) -> Option<&Value> {
        todo!()
    }

    fn delete(&mut self, key: &Key) {
        todo!()
    }
}
