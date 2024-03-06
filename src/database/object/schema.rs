//! # Database Schema Module
//!
//! This module provisions database schematics for fixed-width records, allowing
//! the translation of raw data stored as contiguous sequences of bits to and
//! from meaningful attributes.
//!
//! #### Authorship
//!
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use anyhow::Result;

use crate::database::util;

/* CONSTANTS */

/// The maximum size (in ASCII characters or bytes) of an enumeration variant
/// name corresponding to the data type of an attribute in a schema.
pub const MAX_ENUM_NAME_SIZE: usize = 15;

/* PUBLIC DEFINITIONS */

/// Represents a list of tuples including a name and a size (called attributes),
/// where each name is unique and the size is a number of bits. This is used to
/// "interpret" the raw data within records into meaningful features.
pub struct Schema<'a> {
    attributes: Vec<Attribute<'a>>,
    size: usize,
}

/// Builder pattern intermediary for constructing a schema declaratively out of
/// provided attributes. This is here to help ensure schemas are not changed
/// accidentally after being instantiated.
pub struct SchemaBuilder<'a> {
    attributes: Vec<Attribute<'a>>,
    size: usize,
}

/// Represents a triad of a name string, a size in bits corresponding to an
/// "attribute" or "feature" associated with a database record, and the type
/// of the data it represents.
pub struct Attribute<'a> {
    data: Datatype<'a>,
    name: String,
    size: usize,
}

/// Specifies the type of data being stored within a record within a specific
/// contiguous subset of bits. This is used for interpretation. Here is the
/// meaning of each variant, with its possible sizes in bits:
/// - `ENUM`: Enumeration with size up to 8.
/// - `UINT`: Unsigned integer of arbitrary size.
/// - `SINT`: Signed integer of size greater than 1.
/// - `SPFP`: Single-precision floating point per IEEE 754 of size exactly 32.
/// - `DPFP`: Double-precision floating point per IEEE 754 of size exactly 64.
/// - `CSTR`: C-style string (ASCII character array) of a size divisible by 8.
#[derive(Debug)]
pub enum Datatype<'a> {
    ENUM {
        map: &'a [(u8, [u8; MAX_ENUM_NAME_SIZE]); u8::MAX as usize],
    },
    UINT,
    SINT,
    SPFP,
    DPFP,
    CSTR,
}

/* PRIVATE DEFINITIONS */

struct SchemaIterator<'a> {
    schema: &'a Schema<'a>,
    index: usize,
}

/* PUBLIC INTERFACES */

impl Schema<'_> {
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

impl<'a> Attribute<'_> {
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
    pub fn datatype(&self) -> &'a Datatype<'a> {
        &self.data
    }
}

impl SchemaBuilder<'_> {
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
        self.attributes.push(attr);
        Ok(self)
    }

    /// Constructs the schema using the current state of the `SchemaBuilder`.
    pub fn build(self) -> Schema<'static> {
        Schema {
            attributes: self.attributes,
            size: self.size,
        }
    }
}

impl<'a> Iterator for SchemaIterator<'a> {
    type Item = &'a Attribute<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.schema
            .attributes
            .get(self.index - 1)
    }
}
