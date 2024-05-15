//! # Remoteness (SUR) Record Module
//!
//! TODO
//!
//! #### Authorship
//!
//! - Ishir Garg, 4/3/2024 (ishirgarg@berkeley.edu)

use anyhow::{bail, Context, Result};
use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::slice::BitSlice;
use bitvec::{bitarr, BitArr};

use crate::database::{Attribute, Datatype, Record, Schema, SchemaBuilder};
use crate::model::solver::Remoteness;
use crate::solver::error::SolverError::RecordViolation;
use crate::solver::RecordType;
use crate::util;

/* CONSTANTS */

/// The exact number of bits that are used to encode remoteness.
pub const REMOTENESS_SIZE: usize = 16;

/// The maximum number of bits that can be used to encode a single record.
pub const BUFFER_SIZE: usize = 16;

/* SCHEMA GENERATOR */

/// Return the database table schema associated with a record instance
pub fn schema() -> Result<Schema> {
    let mut schema = SchemaBuilder::new().of(RecordType::REM);

    let name = "State remoteness";
    let data = Datatype::UINT;
    let size = REMOTENESS_SIZE;
    schema = schema
        .add(Attribute::new(name, data, size))
        .context("Failed to add remoteness attribute to database schema.")?;

    Ok(schema.build())
}

/* RECORD IMPLEMENTATION */

/// TODO
pub struct RecordBuffer {
    buf: BitArr!(for BUFFER_SIZE, in u8, Msb0),
}

impl Record for RecordBuffer {
    #[inline(always)]
    fn raw(&self) -> &BitSlice<u8, Msb0> {
        &self.buf[..Self::bit_size()]
    }
}

impl RecordBuffer {
    /// Returns a new instance of a bit-packed record buffer that is able to
    /// store remoteness values
    #[inline(always)]
    pub fn new() -> Result<Self> {
        Ok(Self {
            buf: bitarr!(u8, Msb0; 0; BUFFER_SIZE),
        })
    }

    /// Return a new instance with `bits` as the underlying buffer. Fails in the
    /// event that the size of `bits` is incoherent with the record.
    #[inline(always)]
    pub fn from(bits: &BitSlice<u8, Msb0>) -> Result<Self> {
        let len = bits.len();
        if len > BUFFER_SIZE {
            bail!(RecordViolation {
                name: RecordType::REM.to_string(),
                hint: format!(
                    "The record implementation operates on a buffer of \
                    {BUFFER_SIZE} bits, but there was an attempt to \
                    instantiate one from a buffer of {len} bits.",
                ),
            })
        } else if len < Self::minimum_bit_size() {
            bail!(RecordViolation {
                name: RecordType::REM.to_string(),
                hint: format!(
                    "This record implementation stores remoteness values, but \
                    there was an attempt to instantiate one with from a buffer \
                    with {len} bit(s), which is not enough to store a \
                    remoteness value (which takes {REMOTENESS_SIZE} bits).",
                ),
            })
        } else {
            let mut buf = bitarr!(u8, Msb0; 0; BUFFER_SIZE);
            buf[..len].copy_from_bitslice(bits);
            Ok(Self { buf })
        }
    }

    /* GET METHODS */

    /// Parse and return the remoteness value in the record encoding. Failure
    /// here indicates corrupted state.
    #[inline(always)]
    pub fn get_remoteness(&self) -> Remoteness {
        let start = Self::remoteness_index();
        let end = start + REMOTENESS_SIZE;
        self.buf[start..end].load_be::<Remoteness>()
    }

    /* SET METHODS */

    /// Set this entry to have `value` remoteness. Fails if `value` is too high
    /// to fit in the space dedicated for remoteness within the record.
    #[inline(always)]
    pub fn set_remoteness(&mut self, value: Remoteness) -> Result<()> {
        let size = util::min_ubits(value);
        if size > REMOTENESS_SIZE {
            bail!(RecordViolation {
                name: RecordType::REM.to_string(),
                hint: format!(
                    "This record implementation uses {REMOTENESS_SIZE} bits to \
                    store unsigned integers representing remoteness values, \
                    but there was an attempt to store a remoteness value of \
                    {value}, which requires at least {size} bits to store.",
                ),
            })
        } else {
            let start = Self::remoteness_index();
            let end = start + REMOTENESS_SIZE;
            self.buf[start..end].store_be(value);
            Ok(())
        }
    }

    /* LAYOUT HELPER METHODS */

    /// Return the number of bits that would be needed to store a record
    /// containing utility information for `players` as well as remoteness.
    #[inline(always)]
    const fn bit_size() -> usize {
        REMOTENESS_SIZE
    }

    /// Return the minimum number of bits needed for a valid record buffer.
    #[inline(always)]
    const fn minimum_bit_size() -> usize {
        REMOTENESS_SIZE
    }

    /// Return the bit index of the remoteness entry start in the record buffer.
    #[inline(always)]
    const fn remoteness_index() -> usize {
        0
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// The maximum numeric remoteness value that can be expressed with exactly
    /// REMOTENESS_SIZE bits in an unsigned integer.
    const MAX_REMOTENESS: Remoteness = 2_u64.pow(REMOTENESS_SIZE as u32) - 1;

    #[test]
    fn initialize_from_invalid_buffer_size() {
        let buf1 = bitarr!(u8, Msb0; 0; BUFFER_SIZE + 1);
        let buf2 = bitarr!(u8, Msb0; 0; BUFFER_SIZE + 10);
        let buf3 = bitarr!(u8, Msb0; 0; BUFFER_SIZE + 100);

        assert!(RecordBuffer::from(&buf1).is_err());
        assert!(RecordBuffer::from(&buf2).is_err());
        assert!(RecordBuffer::from(&buf3).is_err());
    }

    #[test]
    fn set_record_attributes() -> Result<()> {
        let mut r1 = RecordBuffer::new().unwrap();

        let good = Remoteness::MIN;
        let bad = Remoteness::MAX;

        r1.set_remoteness(good)?;
        r1.set_remoteness(good)?;
        r1.set_remoteness(good)?;
        assert!(r1.set_remoteness(bad).is_err());
        assert!(r1.set_remoteness(bad).is_err());
        assert!(r1.set_remoteness(bad).is_err());

        Ok(())
    }

    #[test]
    fn data_is_valid_after_round_trip() -> Result<()> {
        let mut record = RecordBuffer::new()?;
        record.set_remoteness(790)?;
        assert_eq!(record.get_remoteness(), 790);
        Ok(())
    }

    #[test]
    fn extreme_data_is_valid_after_round_trip() -> Result<()> {
        let mut record = RecordBuffer::new().unwrap();
        record.set_remoteness(MAX_REMOTENESS)?;
        assert_eq!(record.get_remoteness(), MAX_REMOTENESS);
        Ok(())
    }
}
