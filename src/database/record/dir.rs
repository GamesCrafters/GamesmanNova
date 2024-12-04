//! # Directory Table Record (DTR) Module
//!
//! An implementation of a method of serializing and deserializing information
//! about a database table. Intended to be used as an element of a metadata
//! table within custom database implementations.

use anyhow::{bail, Result};
use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::slice::BitSlice;
use bitvec::{bitarr, BitArr};

use std::ffi::CString;

use crate::database::error::DatabaseError::RecordViolation;
use crate::database::util::SchemaBuilder;
use crate::database::{Attribute, Datatype, Schema};
use crate::util::min_ubits;

/* CONSTANTS */

/// The name of this record implementation, for error reporting purposes.
pub const RECORD_NAME: &str = "Table Directory Record";

/// The number of bits used to store the offset of the page directory header
/// corresponding to a particular table within a database heap file.
pub const PAGE_OFFSET_BITS: usize = 64;

/// The maximum number of ASCII characters (bytes) that can be used for
/// attribute names. This is needed to determine the minimum bits needed to
/// persist attributes' names.
pub const MAX_ATTRIBUTE_NAME_BYTES: usize = 80;

/// The highest number of record data types that can ever be added to the
/// `DataType` enumeration. This is necessary to recover the programmatic
/// representation of an attribute datatype from a serialized source, keeping
/// in mind an extent of backwards and forwards compatibility.
pub const MAX_DATA_TYPE_DISCRIMINANT: usize = 255;

/// The highest number of attributes that can be present in a single table. This
/// needs to be known to fix the directory table record width at a constant.
pub const MAX_SCHEMA_ATTRIBUTES: usize = 127;

/// The maximum bit length of a single attribute. This needs to be known to
/// determine the minimum bits needed to persist attribute sizes.
pub const MAX_ATTRIBUTE_SIZE: usize = 511;

/// The total size of a directory table record in bits. About 10 kB.
pub const BUFFER_SIZE: usize = PAGE_OFFSET_BITS
    + min_ubits(MAX_SCHEMA_ATTRIBUTES as u64)
    + (((MAX_ATTRIBUTE_NAME_BYTES * 8)
        + min_ubits(MAX_ATTRIBUTE_SIZE as u64)
        + min_ubits(MAX_DATA_TYPE_DISCRIMINANT as u64))
        * MAX_SCHEMA_ATTRIBUTES);

/* SCHEMA GENERATOR */

/// Return the database table schema associated with a record instance.
pub fn schema() -> Result<Schema> {
    let mut schema = SchemaBuilder::default()
        .add(Attribute::new(
            "attr_count",
            Datatype::UINT,
            min_ubits(MAX_SCHEMA_ATTRIBUTES as u64),
        ))?
        .add(Attribute::new(
            "byte_offset",
            Datatype::UINT,
            PAGE_OFFSET_BITS,
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
                min_ubits(MAX_DATA_TYPE_DISCRIMINANT as u64),
            ))?;
    }

    Ok(schema.build())
}

/* RECORD IMPLEMENTATION */

#[derive(Default)]
pub struct RecordBuffer {
    buf: BitArr!(for BUFFER_SIZE, in u8, Msb0),
}

impl AsRef<[u8]> for RecordBuffer {
    fn as_ref(&self) -> &[u8] {
        self.buf.as_raw_slice()
    }
}

impl TryFrom<Schema> for RecordBuffer {
    type Error = anyhow::Error;

    fn try_from(schema: Schema) -> Result<RecordBuffer> {
        let mut buf = RecordBuffer::default();
        buf.set_attribute_count(schema.attribute_count())?;

        for (i, attr) in schema
            .attributes()
            .into_iter()
            .enumerate()
        {
            buf.set_attribute(i, attr)?;
        }

        Ok(buf)
    }
}

impl TryFrom<RecordBuffer> for Schema {
    type Error = anyhow::Error;
    fn try_from(buf: RecordBuffer) -> Result<Schema> {
        let mut schema = SchemaBuilder::default();
        for index in 0..buf.get_attribute_count() {
            let attribute = buf.get_attribute(index)?;
            schema = schema.add(attribute)?;
        }
        Ok(schema.build())
    }
}

impl RecordBuffer {
    pub fn new(buf: &BitSlice<u8, Msb0>) -> Result<Self> {
        let length = buf.len();
        if length != BUFFER_SIZE {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "This record implementation uses {BUFFER_SIZE} bits, but \
                    there was an attempt to initialize a record buffer from \
                    another buffer of {length} bits."
                )
            })
        } else {
            let mut record = bitarr!(u8, Msb0; 0; BUFFER_SIZE);
            record[0..BUFFER_SIZE].copy_from_bitslice(buf);
            Ok(Self { buf: record })
        }
    }

    /* GETTERS  */

    /// Retrieve the table offset being stored in this record buffer.
    pub fn get_offset(&self) -> u64 {
        let start = Self::offset_index();
        let end = start + PAGE_OFFSET_BITS;
        self.buf[start..end].load_be()
    }

    /* SETTERS */

    /// Insert a byte offset into this record buffer.
    pub fn set_offset(&mut self, value: u64) -> Result<()> {
        let size = min_ubits(value);
        if size > PAGE_OFFSET_BITS {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "This record implementation uses {PAGE_OFFSET_BITS} bits \
                    to store the offsets of resources within database heap \
                    files, but there was an attempt to store an offset of \
                    {value}, which requires at least {size} bits to store."
                ),
            })
        } else {
            let start = Self::offset_index();
            let end = start + PAGE_OFFSET_BITS;
            self.buf[start..end].store_be(value);
            Ok(())
        }
    }

    /* SETTER HELPERS */

    fn set_attribute_count(&mut self, value: usize) -> Result<()> {
        if value > MAX_SCHEMA_ATTRIBUTES {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "This record implementation can be used to store attribute \
                    counts of up to {MAX_SCHEMA_ATTRIBUTES}, but there was an \
                    attmept to store a record count of {value}, which exceeds \
                    this maximum."
                ),
            })
        } else {
            let start = Self::attribute_count_index();
            let end = start + min_ubits(MAX_SCHEMA_ATTRIBUTES as u64);
            self.buf[start..end].store_be(value);
            Ok(())
        }
    }

    fn set_attribute(
        &mut self,
        index: usize,
        attribute: &Attribute,
    ) -> Result<()> {
        if index > MAX_SCHEMA_ATTRIBUTES - 1 {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "This record implementation can be used to store up to \
                    {MAX_SCHEMA_ATTRIBUTES} attributes, but there was an \
                    attempt to store information at the index of a {index}'th \
                    attribute (0-indexed), which exceeds this maximum."
                ),
            })
        } else {
            self.set_attribute_datatype(index, attribute.datatype())?;
            self.set_attribute_size(index, attribute.size())?;
            self.set_attribute_name(index, attribute.name())?;
            Ok(())
        }
    }

    fn set_attribute_name(&mut self, index: usize, name: &str) -> Result<()> {
        let cstr = CString::new(name)?;
        let bytes = cstr.as_bytes_with_nul();
        let length = bytes.len();
        if length > MAX_ATTRIBUTE_NAME_BYTES {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "This record implementation has {MAX_ATTRIBUTE_NAME_BYTES} \
                    bytes to store attribute names, but there was an attempt \
                    to store an attribute name of byte length {length}."
                ),
            })
        } else {
            let start = Self::attribute_name_index(index);
            let end = start + (MAX_ATTRIBUTE_NAME_BYTES * 8);
            self.buf[start..end]
                .copy_from_bitslice(BitSlice::from_slice(bytes));

            Ok(())
        }
    }

    fn set_attribute_size(&mut self, index: usize, value: usize) -> Result<()> {
        if value > MAX_ATTRIBUTE_SIZE {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "This record implementation can be used to store attribute \
                    sizes of up to {MAX_ATTRIBUTE_SIZE}, but there was an \
                    attempt to store an attribute size of {value}."
                ),
            })
        } else {
            let start = Self::attribute_size_index(index);
            let end = start + min_ubits(MAX_ATTRIBUTE_SIZE as u64);
            self.buf[start..end].store_be(value);
            Ok(())
        }
    }

    fn set_attribute_datatype(
        &mut self,
        index: usize,
        variant: Datatype,
    ) -> Result<()> {
        let value = variant as u8;
        if value as usize > MAX_DATA_TYPE_DISCRIMINANT {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "This record implementation can be used to store attribute \
                    type entries with discriminants of up to \
                    {MAX_DATA_TYPE_DISCRIMINANT}, but there was an attempt \
                    to store an attribute data type with a discriminant of \
                    {value}, which exceeds this maximum."
                ),
            })
        } else {
            let start = Self::attribute_datatype_index(index);
            let end = start + min_ubits(MAX_DATA_TYPE_DISCRIMINANT as u64);
            self.buf[start..end].store_be(value);
            Ok(())
        }
    }

    /* GETTER HELPERS */

    fn get_attribute(&self, index: usize) -> Result<Attribute> {
        Ok(Attribute::new(
            &self.get_attribute_name(index)?,
            self.get_attribute_datatype(index)?,
            self.get_attribute_size(index),
        ))
    }

    fn get_attribute_count(&self) -> usize {
        let start = Self::attribute_count_index();
        let end = start + min_ubits(MAX_SCHEMA_ATTRIBUTES as u64);
        self.buf[start..end].load_be()
    }

    fn get_attribute_datatype(&self, index: usize) -> Result<Datatype> {
        let start = Self::attribute_datatype_index(index);
        let end = start + min_ubits(MAX_DATA_TYPE_DISCRIMINANT as u64);
        let discriminant = self.buf[start..end].load_be::<u8>();
        discriminant.try_into()
    }

    fn get_attribute_size(&self, index: usize) -> usize {
        let start = Self::attribute_size_index(index);
        let end = start + min_ubits(MAX_ATTRIBUTE_SIZE as u64);
        self.buf[start..end].load_be()
    }

    fn get_attribute_name(&self, index: usize) -> Result<String> {
        let start = Self::attribute_name_index(index);
        let end = start + (MAX_ATTRIBUTE_NAME_BYTES * 8);
        let mut bits = self.buf[start..end].to_bitvec();
        bits.force_align();

        let bytes = bits.into_vec();
        let cstr = CString::new(bytes)?;
        let str = cstr.into_string()?;
        Ok(str)
    }

    /* INDEX HELPERS */

    /// Return the bit index of an attribute count in the record buffer.
    #[inline(always)]
    const fn attribute_count_index() -> usize {
        0
    }

    /// Return the bit index of a resource offset in the record buffer.
    #[inline(always)]
    const fn offset_index() -> usize {
        Self::attribute_count_index() + min_ubits(MAX_SCHEMA_ATTRIBUTES as u64)
    }

    /// Return the bit index of the `i`'th attribute's name.
    #[inline(always)]
    const fn attribute_name_index(i: usize) -> usize {
        Self::attribute_entry_start(i)
    }

    /// Return the bit index of the `i`'th attribute's size.
    #[inline(always)]
    const fn attribute_size_index(i: usize) -> usize {
        Self::attribute_name_index(i) + (MAX_ATTRIBUTE_NAME_BYTES * 8)
    }

    /// Return the bit index of the `i`'th attribute's datatype.
    #[inline(always)]
    const fn attribute_datatype_index(i: usize) -> usize {
        Self::attribute_size_index(i) + min_ubits(MAX_ATTRIBUTE_SIZE as u64)
    }

    /// Return the bit index of the start of the `i`'th attribute.
    #[inline(always)]
    const fn attribute_entry_start(i: usize) -> usize {
        let base = Self::offset_index() + PAGE_OFFSET_BITS;
        let entry_width = (MAX_ATTRIBUTE_NAME_BYTES * 8)
            + min_ubits(MAX_DATA_TYPE_DISCRIMINANT as u64)
            + min_ubits(MAX_ATTRIBUTE_SIZE as u64);

        base + (i * entry_width)
    }
}

#[cfg(test)]
mod test {
    // TODO
}
