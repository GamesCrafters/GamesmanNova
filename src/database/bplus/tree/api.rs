 //! # Bplus Tree API [WIP]
//!
//! This module provides a bplus tree API to build trees
//! and perform insert, delete, and lookup operations.

/* IMPORTS */

use crate::{
    database::{bplus::tree::Error, Record},
    model::State,
};

/* CONSTANTS */

/* DEFINITIONS */

pub struct BTree<'a> {
    todo: &'a str,
}

pub struct BTreeBuilder<'a> {
    todo: &'a str,
}

/* IMPLEMENTATIONS */

impl BTree<'_> {
    fn insert(&mut self, record: &Record) -> Result<(), Error> {
        todo!()
    }

    fn delete(&mut self, state: &State) -> Result<(), Error> {
        todo!()
    }

    fn search(&mut self, state: &State) -> Result<Record, Error> {
        todo!()
    }

    fn print(&mut self) -> Result<(), Error> {
        todo!()
    }
}

impl BTreeBuilder<'_> {
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