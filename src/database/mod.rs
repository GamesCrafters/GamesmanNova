//! # Database Module
//!
//! This module contains memory and I/O mechanisms used to store and fetch
//! analysis data generically, hopefully in an efficient and scalable way.

use anyhow::Result;

use std::path::Path;

/* RE-EXPORTS */

pub use util::SchemaBuilder;

/* UTILITY MODULES */

#[cfg(test)]
mod test;
mod util;

pub mod model;
pub mod error;

/* IMPLEMENTATION MODULES */

/// TODO
pub mod engine {
    pub mod sled;
}

/// TODO
pub mod record {
    pub mod mur;
    pub mod dir;
}

/* DEFINITIONS */

/// Represents a list of tuples including a name and a size (called attributes),
/// where each name is unique and the size is a number of bits. This is used to
/// "interpret" the raw data within records into meaningful features.
#[derive(Clone)]
pub struct Schema {
    attribute_count: usize,
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
/// - `BOOL`: Boolean of size exactly 1.
/// - `ENUM`: Enumeration of arbitrary size.
/// - `UINT`: Unsigned integer of arbitrary size.
/// - `SINT`: Signed integer of size greater than 1.
/// - `SPFP`: Single-precision floating point per IEEE 754 of size exactly 32.
/// - `DPFP`: Double-precision floating point per IEEE 754 of size exactly 64.
/// - `CSTR`: C-style string (ASCII character array) of a size divisible by 8.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Datatype {
    BOOL,
    ENUM,
    UINT,
    SINT,
    SPFP,
    DPFP,
    CSTR,
}

/* DATABASE RESOURCE INTERFACES */

/// Allows a resource to act as a map from bytes to other bytes. Commonly
/// understood as a Key-Value store.
pub trait ByteMap {
    /// Replaces the value associated with `key` with the bytes of `record`,
    /// creating one if it does not already exist. Fails if under any violation
    /// of implementation-specific assumptions of record size or contents.
    fn insert<K, V>(&mut self, key: K, record: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>;

    /// Returns the bytes associated with the value of `key`, or `None` if there
    /// is no such association.
    fn get<K>(&self, key: K) -> Result<Option<Vec<u8>>>
    where
        K: AsRef<[u8]>;

    /// Removes the association of `key` to whatever value it is currently bound
    /// to, or does nothing if there is no such value.
    fn remove<K>(&mut self, key: K) -> Result<Option<Vec<u8>>>
    where
        K: AsRef<[u8]>;
}

/// Allows a database to be sectionable into different namespaces, such as
/// tables in the case of a relational database. Such sections are assumed to
/// operate over their elements independently of each other.
pub trait ProtoRelational {
    type Namespace: ByteMap + Relation;

    /// Create and return a new namespace under this database enforcing the
    /// provided `schema` as a relation.
    fn namespace(&self, schema: Schema, name: &str) -> Result<Self::Namespace>;

    /// Drop the namespace with `name` under this database. Returns true if
    /// this in fact dropped a namespace.
    fn drop(&self, name: &str) -> Result<bool>;
}

/// Allows a database to be evicted to persistent media. Implementing this trait
/// requires custom handling of what happens when the database is closed; if it
/// has data on memory, then it should persist dirty data to ensure consistency.
/// This is possible via [`Persistent::flush`].
pub trait Persistent {
    /// Interprets the contents of a directory at `path` to be the contents of
    /// a persistent database. If no contents have been written, they will be
    /// initialized. Fails if the path does not exist.
    fn new(path: &Path) -> Result<Self>
    where
        Self: Sized;

    /// Persist the in-memory contents of the database to disk. Potentially very
    /// slow. Use sparingly to ensure recoverability.
    fn flush(&self) -> Result<usize>;
}

/// A grouping of fixed-length records which share a table [`Schema`] that can
/// be used as a handle to mutate them via [`Map`] semantics, in addition to
/// keeping track of useful metadata.
pub trait Relation
where
    Self: ByteMap,
{
    /// Returns a reference to the schema associated with `self`.
    fn schema(&self) -> &Schema;

    /// Returns the number of records currently contained in `self`.
    fn count(&self) -> usize;
}

/* IMPLEMENTATIONS */

impl Schema {
    /// Returns the sum of the sizes of the schema's attributes.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the number of attributes in the schema.
    pub fn attribute_count(&self) -> usize {
        self.attribute_count
    }

    /// Returns the attributes contained in this schema.
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
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
