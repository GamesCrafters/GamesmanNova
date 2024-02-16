//! # Database Module [WIP]
//!
//! This module contains memory and I/O mechanisms used to store and fetch
//! solution set data, hopefully in an efficient and scalable way.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

use crate::error::NovaError;
use std::path::Path;

pub mod schema;
pub mod simple;

/* DATABASE PARAMETERS */

pub enum Persistence<'a> {
    On(&'a Path),
    Off,
}

/* INTERFACE DEFINITIONS */

pub trait KVStore {
    fn put(&mut self, key: usize, value: &[u8]);
    fn get(&self, key: usize) -> Option<&[u8]>;
    fn delete(&self, key: usize);
}

/* FEATURE ADDITIONS */

pub trait Persistent
where
    Self: Drop,
{
    fn bind(&self, path: &Path) -> Result<(), NovaError>;
}

pub trait Tabular {
    fn create_table(&self, id: &str, width: u32) -> Result<(), NovaError>;
    fn select_table(&self, id: &str) -> Result<(), NovaError>;
    fn delete_table(&self, id: &str) -> Result<(), NovaError>;
}
