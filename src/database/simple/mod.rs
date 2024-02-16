//! # Simple Database
//!
//! This module contains a very simple implementation of a persistent in-memory
//! key-value store. It works by indexing into an allocated vector through keys,
//! always making sure that it is large enough to house the record with the
//! highest key. This means that its top capacity is the amount of memory that
//! can be allocated by the operating system, without considering the usage of
//! virtual memory.
//!
//! For persistence, a file is created containing a bit-accurate representation
//! of the in-memory vector. Table logic is handled by switching which of these
//! files is currently being targeted, with the understanding that the contents
//! of memory are materialized every time there is a table switch.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

use std::fs::File;

use crate::database::Persistence;
use crate::database::{KVStore, Tabular};
use crate::error::NovaError;

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

impl Database<'_> {
    fn initialize(params: Parameters) -> Result<Self, NovaError> {
        let mode = params.persistence;
        let buffer = Vec::new();
        let table = Table {
            dirty: false,
            width: 0,
            name: METADATA_TABLE,
            size: 0,
        };

        if let Persistence::On(path) = params.persistence {
            assert!(path.exists() && path.is_dir());
            let path = path.join(METADATA_TABLE);
            let meta = if !path.is_file() {
                let f = File::create(path).unwrap();
                initialize_metadata_table(f)?;
                f
            } else {
                File::open(path).unwrap()
            };
        }

        Ok(Database {
            mode,
            buffer,
            table,
        })
    }
}

impl KVStore for Database<'_> {
    fn put(&mut self, key: usize, value: &[u8]) {
        self.buffer.insert(key, value)
    }

    fn get(&self, key: usize) -> Option<&[u8]> {
        todo!()
    }

    fn delete(&self, key: usize) {
        todo!()
    }
}

impl Tabular for Database<'_> {
    fn create_table(&self, id: &str, width: u32) -> Result<(), NovaError> {
        todo!()
    }

    fn select_table(&self, id: &str) -> Result<(), NovaError> {
        todo!()
    }

    fn delete_table(&self, id: &str) -> Result<(), NovaError> {
        todo!()
    }
}

fn initialize_metadata_table(file: File) -> Result<(), NovaError> {
    todo!()
}
