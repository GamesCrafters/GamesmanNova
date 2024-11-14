//! # Directory Table Record (DTR) Module
//!
//! TODO

use anyhow::Result;
use bitvec::order::Msb0;
use bitvec::slice::BitSlice;
use bitvec::{bitarr, BitArr};

use crate::database::util::SchemaBuilder;
use crate::database::{Attribute, Datatype, Record, RecordType, Schema};
use crate::util::min_ubits;

/* CONSTANTS */

/// The maximum number of ASCII characters (bytes) that can be used for
/// attribute names. This is needed to determine the minimum bits needed to
/// persist attributes' names.
pub const MAX_ATTRIBUTE_NAME_BYTES: usize = 80;

/// The highest number of record data types that can ever be added to the
/// `DataType` enumeration. This is necessary to recover the programmatic
/// representation of an attribute datatype from a serialized source, keeping
/// in mind an extent of backwards and forwards compatibility.
pub const MAX_SUPPORTED_DATA_TYPES: usize = 1024;

/// The highest number of record type variants that can ever be added to the
/// `RecordType` enumeration. This is necessary to recover the programmatic
/// representation of a table record type from a serialized source, keeping in
/// mind an extent of backwards and forwards compatibility.
pub const MAX_SUPPORTED_RECORD_TYPES: usize = 1024;

/// The maximum number of ASCII characters (bytes) that can be used for a table
/// name (including a null terminator).
pub const MAX_TABLE_NAME_BYTES: usize = 80;

/// The highest number of attributes that can be present in a single table. This
/// needs to be known to fix the directory table record width at a constant.
pub const MAX_SCHEMA_ATTRIBUTES: usize = 128;

/// The maximum bit length of a single attribute. This needs to be known to
/// determine the minimum bits needed to persist attribute sizes.
pub const MAX_ATTRIBUTE_SIZE: usize = 512;

/// The total size of a directory table record in bits. About 10 kB.
pub const BUFFER_SIZE: usize = (MAX_TABLE_NAME_BYTES * 8)
    + min_ubits(MAX_SUPPORTED_RECORD_TYPES as u64)
    + min_ubits(MAX_SCHEMA_ATTRIBUTES as u64)
    + (((MAX_ATTRIBUTE_NAME_BYTES * 8)
        + min_ubits(MAX_ATTRIBUTE_SIZE as u64)
        + min_ubits(MAX_SUPPORTED_DATA_TYPES as u64))
        * MAX_SCHEMA_ATTRIBUTES);

/* SCHEMA GENERATOR */

/// Return the database table schema associated with a record instance.
pub fn schema(name: &str) -> Result<Schema> {
    let mut schema = SchemaBuilder::new(name)
        .of(RecordType::DTR)
        .add(Attribute::new(
            "table_name",
            Datatype::CSTR,
            MAX_TABLE_NAME_BYTES * 8,
        ))?
        .add(Attribute::new(
            "record_type",
            Datatype::ENUM,
            min_ubits(MAX_SUPPORTED_RECORD_TYPES as u64),
        ))?
        .add(Attribute::new(
            "attr_count",
            Datatype::UINT,
            min_ubits(MAX_SCHEMA_ATTRIBUTES as u64),
        ))?;

    for i in 0..MAX_SCHEMA_ATTRIBUTES {
        let name_prefix = format!("attr{}", i);
        schema = schema
            .add(Attribute::new(
                &format!("{}_name", name_prefix),
                Datatype::CSTR,
                MAX_ATTRIBUTE_NAME_BYTES * 8,
            ))?
            .add(Attribute::new(
                &format!("{}_size", name_prefix),
                Datatype::UINT,
                min_ubits(MAX_ATTRIBUTE_SIZE as u64),
            ))?
            .add(Attribute::new(
                &format!("{}_type", name_prefix),
                Datatype::ENUM,
                min_ubits(MAX_SUPPORTED_DATA_TYPES as u64),
            ))?;
    }

    Ok(schema.build())
}

/* RECORD IMPLEMENTATION */

/// TODO
pub struct RecordBuffer {
    buf: BitArr!(for BUFFER_SIZE, in u8, Msb0),
}

impl RecordBuffer {
    fn new() -> Self {
        Self {
            buf: bitarr!(u8, Msb0; 0; BUFFER_SIZE),
        }
    }
}

impl Record for RecordBuffer {
    #[inline(always)]
    fn raw(&self) -> &BitSlice<u8, Msb0> {
        &self.buf
    }
}

impl TryInto<RecordBuffer> for Schema {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<RecordBuffer> {
        let mut buf = RecordBuffer::new();
        buf.set_datatype(self.datatype())?;
        buf.set_table_name(&self.table_name())?;
        buf.set_attribute_count(self.attribute_count())?;

        for attr in self.attributes() {
            buf.set_attribute(attr)?;
        }

        Ok(buf)
    }
}

impl TryInto<Schema> for RecordBuffer {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Schema> {
        todo!()
    }
}

impl RecordBuffer {
    fn set_table_name(&mut self, name: &str) -> Result<()> {
        todo!()
    }

    fn set_datatype(&mut self, variant: Option<RecordType>) -> Result<()> {
        todo!()
    }

    fn set_attribute_count(&mut self, count: usize) -> Result<()> {
        todo!()
    }

    fn set_attribute(&mut self, attribute: &Attribute) -> Result<()> {
        todo!()
    }
}
