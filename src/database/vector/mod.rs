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

use anyhow::Result;

use std::path::Path;

use crate::{
    database::model::{Key, SequenceKey, Value},
    database::{self, KVStore, Persistent, Record, Schema},
};

/* DEFINITIONS */

pub struct Database {}

pub struct Table {}

/* IMPLEMENTATIONS */

impl Persistent for Database {
    fn from(path: &Path) -> Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn bind(&self, path: &Path) -> Result<()> {
        todo!()
    }
}

impl Drop for Database {
    fn drop(&mut self) {
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

    fn bytes(&self) -> u64 {
        todo!()
    }

    fn id(&self) -> SequenceKey {
        todo!()
    }
}

impl KVStore for Table {
    fn insert<R>(&mut self, key: &Key, value: &R) -> Result<()>
    where
        R: Record,
    {
        todo!()
    }

    fn get(&self, key: &Key) -> Option<&Value> {
        todo!()
    }

    fn remove(&mut self, key: &Key) {
        todo!()
    }
}
