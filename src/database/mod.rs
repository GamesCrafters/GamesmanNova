#![allow(drop_bounds)]
//! # Database Module
//!
//! This module contains memory and I/O mechanisms used to store and fetch
//! analysis data, hopefully in an efficient and scalable way.

use anyhow::Result;

use std::path::{Path, PathBuf};

use crate::database::model::{Key, SequenceKey, Value};
use crate::solver::RecordType;

/* RE-EXPORTS */

pub use util::SchemaBuilder;

/* UTILITY MODULES */

#[cfg(test)]
mod test;
mod util;

pub mod model;
pub mod error;

/* IMPLEMENTATION MODULES */

pub mod volatile;
pub mod vector;
pub mod lsmt;

/* DEFINITIONS */

/// Indicates whether the database implementation should store the data it is
/// managing to disk, or keep it entirely in memory.
pub enum Persistence {
    On(PathBuf),
    Off,
}

/// Represents a list of tuples including a name and a size (called attributes),
/// where each name is unique and the size is a number of bits. This is used to
/// "interpret" the raw data within records into meaningful features.
pub struct Schema {
    attributes: Vec<Attribute>,
    record: Option<RecordType>,
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
pub enum Datatype {
    BOOL,
    ENUM,
    UINT,
    SINT,
    SPFP,
    DPFP,
    CSTR,
}

/* DATABASE INTERFACES */

/// Represents the behavior of a Key-Value Store generic over a [`Record`] type.
pub trait KVStore {
    /// Replaces the value associated with `key` with the bits of `record`,
    /// creating one if it does not already exist. Fails if under any violation
    /// of implementation-specific assumptions of record size or contents.
    fn insert<R: Record>(&mut self, key: &Key, record: &R) -> Result<()>;

    /// Returns the bits associated with the value of `key`, or `None` if there
    /// is no such association. Infallible due to all possible values of `key`
    /// being considered valid (but not necessarily existent).
    fn get(&self, key: &Key) -> Option<&Value>;

    /// Removes the association of `key` to whatever value it is currently bound
    /// to, or does nothing if there is no such value.
    fn remove(&mut self, key: &Key);
}

/// Allows a database to be evicted to persistent media. Implementing this trait
/// requires custom handling of what happens when the database is closed; if it
/// has data on memory, then it should persist dirty data to ensure consistency
/// via [`Drop`]. Database file structure is implementation-specific.
pub trait Persistent<T>
where
    Self: Tabular<T> + Drop,
    T: Table,
{
    /// Interprets the contents of a directory at `path` to be the contents of
    /// a persistent database. Fails if the contents of `path` are unexpected.
    fn from(path: &Path) -> Result<Self>
    where
        Self: Sized;

    /// Binds the contents of the database to a particular `path` for the sake
    /// of persistence. It is undefined behavior to forego calling this function
    /// before pushing data to the underlying database. Fails if the database is
    /// already bound to another path, or if `path` is non-empty, or under any
    /// I/O failure.
    fn bind(&self, path: &Path) -> Result<()>;

    /// Evict the contents of `table` to disk in a batch operation, potentially
    /// leaving cache space for other table's usage. Calling this on all tables
    /// in a database should be equivalent to dropping the database reference.
    fn flush(&self, table: &mut T) -> Result<()>;
}

/// Allows for grouping data into [`Table`] implementations, which contain many
/// fixed-length records that share attributes under a single [`Schema`]. This
/// allows consumers of this implementation to have simultaneous references to
/// different mutable tables.
pub trait Tabular<T>
where
    T: Table,
{
    /// Creates a new table with `schema`. Returns a unique key that can be used
    /// to later acquire the table.
    fn insert_table(&self, schema: Schema) -> Result<SequenceKey>;

    /// Obtains a mutable reference to the [`Table`] with `id`. Fails if no such
    /// table exists in the underlying database, or under any I/O failure.
    fn get_table_mut(&self, key: SequenceKey) -> Result<&mut T>;

    /// Obtains an immutable reference to the [`Table`] with `id`. Fails if no
    /// such table exists in the underlying database, or under any I/O failure.
    fn get_table(&self, key: SequenceKey) -> Result<&T>;

    /// Forgets about the association of `id` to any existing table, doing
    /// nothing if there is no such table. Fails under any I/O failure.
    fn remove_table(&self, table: &mut T) -> Result<()>;
}

/* TABLE INTERFACE */

/// A grouping of fixed-length records which share a table [`Schema`] that can
/// be used as a handle to mutate them via [`KVStore`] semantics, in addition
/// to keeping track of useful metadata.
pub trait Table
where
    Self: KVStore,
{
    /// Returns a reference to the schema associated with `self`.
    fn schema(&self) -> &Schema;

    /// Returns the number of records currently contained in `self`.
    fn count(&self) -> u64;

    /// Returns the total number of bytes being used to store the contents of
    /// `self`, excluding metadata (both in memory and persistent media).
    fn bytes(&self) -> u64;

    /// Returns the identifier associated with `self`.
    fn id(&self) -> SequenceKey;
}

/* RECORD INTERFACE */

/// Represents an in-memory sequence of bits that can be directly accessed.
pub trait Record {
    /// Returns a reference to the sequence of bits in `self`.
    fn raw(&self) -> &Value;
}

/* IMPLEMENTATIONS */

impl Schema {
    /// Returns the sum of the sizes of the schema's attributes.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the record type associated with this schema, if any.
    pub fn record(&self) -> Option<RecordType> {
        self.record
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
