//! # Database Module [WIP]
//!
//! This module contains memory and I/O mechanisms used to store and fetch
//! solution set data, hopefully in an efficient and scalable way.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

use anyhow::Result;
use bitvec::prelude::{BitSlice, Msb0};

use std::path::Path;

use crate::model::State;

/* UTILITY MODULES */

mod error;
mod util;

/* IMPLEMENTATION MODULES */

pub mod volatile;
pub mod vector;
pub mod lsmt;

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
pub trait KVStore<R: Record> {
    fn put(&mut self, key: State, record: &R);
    fn get(&self, key: State) -> Option<&BitSlice<u8, Msb0>>;
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
    fn create_table(&self, id: &str, schema: Schema) -> Result<()>;
    fn select_table(&self, id: &str) -> Result<()>;
    fn delete_table(&self, id: &str) -> Result<()>;
}

/// Allows a database implementation to read raw data from a record buffer.
pub trait Record {
    fn raw(&self) -> &BitSlice<u8, Msb0>;
}

/* SCHEMA DEFINITIONS */

/// Represents a list of tuples including a name and a size (called attributes),
/// where each name is unique and the size is a number of bits. This is used to
/// "interpret" the raw data within records into meaningful features.
pub struct Schema {
    attributes: Vec<Attribute>,
    size: usize,
}

/// Builder pattern intermediary for constructing a schema declaratively out of
/// provided attributes. This is here to help ensure schemas are not changed
/// accidentally after being instantiated.
pub struct SchemaBuilder {
    attributes: Vec<Attribute>,
    size: usize,
}

/// Represents a triad of a name string, a size in bits corresponding to an
/// "attribute" or "feature" associated with a database record, and the type
/// of the data it represents.
#[derive(Clone)]
pub struct Attribute {
    data: Datatype,
    name: String,
    size: usize,
}

/// Specifies the type of data being stored within a record within a specific
/// contiguous subset of bits. This is used for interpretation. Here is the
/// meaning of each variant, with its possible sizes in bits:
/// - `ENUM`: Enumeration of arbitrary size.
/// - `UINT`: Unsigned integer of arbitrary size.
/// - `SINT`: Signed integer of size greater than 1.
/// - `SPFP`: Single-precision floating point per IEEE 754 of size exactly 32.
/// - `DPFP`: Double-precision floating point per IEEE 754 of size exactly 64.
/// - `CSTR`: C-style string (ASCII character array) of a size divisible by 8.
#[derive(Debug, Copy, Clone)]
pub enum Datatype {
    ENUM,
    UINT,
    SINT,
    SPFP,
    DPFP,
    CSTR,
}

pub struct SchemaIterator<'a> {
    schema: &'a Schema,
    index: usize,
}

impl Schema {
    /// Returns the sum of the sizes of the schema's attributes.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns an iterator over the attributes in the schema.
    pub fn iter(&self) -> SchemaIterator {
        SchemaIterator {
            schema: &self,
            index: 0,
        }
    }
}

impl Attribute {
    /// Returns a new `Attribute` with `name` and `size`.
    pub fn new(name: &str, data: Datatype, size: usize) -> Self {
        Attribute {
            data,
            name: name.to_owned(),
            size,
        }
    }

    /// Returns the name associated with the attribute.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the size (in bits) of this attribute.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the data type of this attribute.
    pub fn datatype(&self) -> Datatype {
        self.data
    }
}

impl SchemaBuilder {
    /// Returns a new instance of a `SchemaBuilder`, which can be used to
    /// declaratively construct a new record `Schema`.
    pub fn new() -> Self {
        SchemaBuilder {
            attributes: Vec::new(),
            size: 0,
        }
    }

    /// Associates `attr` to the schema under construction. Returns an error
    /// if adding `attr` to the schema would result in an invalid state.
    pub fn add(mut self, attr: Attribute) -> Result<Self> {
        util::check_attribute_validity(&self.attributes, &attr)?;
        self.size += attr.size();
        Ok(self)
    }

    /// Constructs the schema using the current state of the `SchemaBuilder`.
    pub fn build(self) -> Schema {
        Schema {
            attributes: self.attributes,
            size: self.size,
        }
    }
}

impl<'a> Iterator for SchemaIterator<'a> {
    type Item = &'a Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.schema
            .attributes
            .get(self.index - 1)
    }
}
