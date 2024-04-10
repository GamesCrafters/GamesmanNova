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
use bitvec::order::Msb0;
use bitvec::slice::BitSlice;

use crate::database::Persistence;
use crate::database::Schema;
use crate::database::{KVStore, Record, Tabular};
use crate::model::State;

/* CONSTANTS */

const METADATA_TABLE: &'static str = ".metadata";

/* DATABASE DEFINITION */

pub struct Database<'a> {
    buffer: Vec<u8>,
    table: Table<'a>,
    mode: Persistence<'a>,
}

struct Table<'a> {
    dirty: bool,
    width: u32,
    name: &'a str,
    size: u128,
}

pub struct Parameters<'a> {
    persistence: Persistence<'a>,
}

/* IMPLEMENTATION */

impl Database<'_> {
    fn initialize(params: Parameters) -> Result<Self> {
        todo!()
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
