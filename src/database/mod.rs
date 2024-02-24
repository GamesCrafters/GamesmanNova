//! # Database Module [WIP]
//!
//! This module contains memory and I/O mechanisms used to store and fetch
//! solution set data, hopefully in an efficient and scalable way.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

use anyhow::Result;

use std::path::Path;

use crate::model::{RecordLength, State};

/* UTILITY MODULES */

mod object;
mod error;
mod util;

/* IMPLEMENTATION MODULE */

pub mod engine;

/* DATABASE PARAMETERS */

/// Indicates whether the database implementation should store the data it is
/// managing to disk, or keep it entirely in memory.
pub enum Persistence<'a> {
    On(&'a Path),
    Off,
}

/* INTERFACE DEFINITIONS */

/// Represents the behavior of a Key-Value Store. No assumptions are made about
/// the size of the records being used, but keys are taken to be fixed-length.
pub trait KVStore {
    fn put(&mut self, key: State, value: &[u8]);
    fn get(&self, key: State) -> Option<&[u8]>;
    fn del(&self, key: State);
}

/* FEATURE ADDITIONS */

/// Allows a database to be evicted to persistent media. Implementing this trait
/// requires custom handling of what happens when the database is closed; if it
/// has data on memory, then it should persist any dirty pages to ensure
/// consistency. In terms of file structure, each implementation decides how to
/// organize its persistent content. The only overarching requisite is that it
/// be provided an existing directory's path.
pub trait Persistent {
    fn bind_path(&self, path: &Path) -> Result<()>;
    fn materialize(&self) -> Result<()>;
}

/// Allows for grouping data into collections of fixed-length records called
/// tables. Because of this application's requirements, this does not mean that
/// a database should be optimized for inter-table operations. In fact, this
/// interface's semantics are such that its implementations optimize performance
/// for cases of sequential operations on a single table.
pub trait Tabular {
    fn create_table(&self, id: &str, width: RecordLength) -> Result<()>;
    fn select_table(&self, id: &str) -> Result<()>;
    fn delete_table(&self, id: &str) -> Result<()>;
}
