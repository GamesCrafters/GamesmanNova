//! # Bplus Database [WIP]
//!
//! This module provides a database implementation backed by a
//! persistent bplus tree.

/* IMPORTS */

use anyhow::Result;
use bitvec::prelude::{BitSlice, Msb0};

use std::path::Path;

use crate::{
    database::{Persistence, KVStore, Persistent, Tabular, Record, Schema},
    model::State,
};

/* CONSTANTS */

/* DEFINITIONS */

pub struct Database<'a> {
    todo: &'a str,
}

pub struct Parameters<'a> {
    persistence: Persistence<'a>,
}

/* IMPLEMENTATIONS */

impl Database<'_> {
    pub fn initialize(params: Parameters) -> Result<Self> {
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

impl Persistent for Database<'_> {
    fn bind_path(&self, path: &Path) -> Result<()> {
        todo!()
    }

    fn materialize(&self) -> Result<()> {
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

/* UNIT TESTING */

#[cfg(test)]
mod tests {
    use super::*;
}