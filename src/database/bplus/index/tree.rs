//! # Bplus Tree Implementation [WIP]
//!
//! This file contains a b+ tree implementation
//! and a b+ tree builder to help with construction.

/* IMPORTS */

use crate::{
    database::{
        bplus::index::{error::Error, page_manager::PageManager},
        Record,
    },
    model::State,
};
use anyhow::Result;
use std::path::Path;

/* DEFINITIONS */

pub struct BTree {
    order: usize,
    page_manager: PageManager,
}

pub struct BTreeBuilder {
    order: usize,
    root: &'static Path,
}

/* IMPLEMENTATIONS */

impl BTree {
    fn insert<R: Record>(&mut self, record: R) -> Result<(), Error> {
        todo!()
    }

    fn delete(&mut self, state: &State) -> Result<(), Error> {
        todo!()
    }

    fn lookup<R: Record>(&mut self, state: &State) -> Result<R, Error> {
        todo!()
    }

    #[cfg(debug_assertions)]
    fn print(&mut self) -> Result<(), Error> {
        todo!()
    }
}

impl BTreeBuilder {
    pub fn initialize() -> Result<Self> {
        Ok(BTreeBuilder {
            order: 0,
            root: Path::new(""),
        })
    }

    pub fn set_order(mut self, order: usize) -> BTreeBuilder {
        self.order = order;
        self
    }

    pub fn set_root(mut self, root: &'static Path) -> BTreeBuilder {
        self.root = root;
        self
    }

    pub fn build(&self) -> Result<BTree, Error> {
        todo!()
    }
}

/* UNIT TESTING */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_builder() -> Result<()> {
        let _ = BTreeBuilder::initialize()?;
        Ok(())
    }

    #[test]
    fn set_builder_fields() -> Result<()> {
        let builder = BTreeBuilder::initialize()?
            .set_order(5)
            .set_root(Path::new("/db"));
        assert_eq!(5, builder.order, "incorrect builder order");
        match builder.root.to_str() {
            None => panic!("new path is not a valid UTF-8 sequence"),
            Some(s) => assert_eq!("/db", s, "incorrect builder root"),
        };
        Ok(())
    }
}
