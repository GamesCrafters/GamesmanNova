//! # Bplus Tree Implementation [WIP]
//!
//! This file contains a b+ tree implementation
//! and a b+ tree builder to help with construction.

/* IMPORTS */

use crate::{
    database::{bplus::index::error::Error, Record},
    model::State,
};
use anyhow::Result;

/* DEFINITIONS */

struct BTreeRecord {}

pub struct BTree {}

pub struct BTreeBuilder {}

/* IMPLEMENTATIONS */

impl Record for BTreeRecord {
    fn raw(&self) -> &bitvec::prelude::BitSlice<u8, bitvec::prelude::Msb0> {
        todo!()
    }
}

impl BTree {
    fn insert(&mut self, record: BTreeRecord) -> Result<(), Error> {
        todo!()
    }

    fn delete(&mut self, state: &State) -> Result<(), Error> {
        todo!()
    }

    fn lookup(&mut self, state: &State) -> Result<BTreeRecord, Error> {
        todo!()
    }

    #[cfg(debug_assertions)]
    fn print(&mut self) -> Result<(), Error> {
        todo!()
    }
}

impl BTreeBuilder {
    pub fn initialize() -> Result<Self> {
        todo!()
    }

    pub fn build(&self) -> Result<BTree, Error> {
        todo!()
    }
}

/* UNIT TESTING */

#[cfg(test)]
mod tests {
    use super::*;
}
