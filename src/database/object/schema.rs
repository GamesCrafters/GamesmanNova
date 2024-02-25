//! # Database Schema Module
//!
//! This module provisions database schematics for fixed-width records, allowing
//! the translation of raw data stored as contiguous sequences of bytes to and
//! from meaningful attributes.
//!
//! #### Authorship
//!
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use anyhow::Result;

use crate::database::util;

/* PUBLIC DEFINITIONS */

/// Represents a list of tuples including a name and a size (called attributes),
/// where each name is unique and the size is a number of bytes. This is used to
/// "interpret" the raw data within records into meaningful features.
pub struct Schema {
    attributes: Vec<Attribute>,
    size: u64,
}

/// Builder pattern intermediary for constructing a schema declaratively out of
/// provided attributes. This is here to help ensure schemas are not changed
/// accidentally after being instantiated.
pub struct SchemaBuilder {
    attributes: Vec<Attribute>,
    size: u64,
}

/// Represents a tuple of a name string and a size in bytes corresponding to an
/// "attribute" or "feature" associated with a database record.
pub struct Attribute {
    name: String,
    size: u32,
}

/* PRIVATE DEFINITIONS */

struct SchemaIterator<'a> {
    schema: &'a Schema,
    index: usize,
}

/* PUBLIC INTERFACES */

impl Schema {
    /// Returns the sum of the sizes of the schema's attributes.
    pub fn size(&self) -> u64 {
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
    pub fn new(name: &str, size: u32) -> Self {
        Attribute {
            name: name.to_string(),
            size,
        }
    }

    /// Returns the name associated with the attribute.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the size (in bytes) of this attribute.
    pub fn size(&self) -> u32 {
        self.size
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
        self.size += <u32 as Into<u64>>::into(attr.size());
        self.attributes.push(attr);
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
