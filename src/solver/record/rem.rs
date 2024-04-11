//! # Remoteness (SUR) Record Module
//!
//! Implementation of a database record buffer for storing only remoteness values
//! Note that this record does not store any utilities, making it useful for puzzles
//!
//! #### Authorship
//!
//! - Ishir Garg, 4/3/2024 (ishirgarg@berkeley.edu)

use anyhow::{Context, Result};
use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::slice::BitSlice;
use bitvec::{bitarr, BitArr};

use crate::database::{Attribute, Datatype, Record, Schema, SchemaBuilder};
use crate::model::Remoteness;
use crate::solver::error::SolverError::RecordViolation;
use crate::solver::util;
use crate::solver::RecordType;

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

/// Solver-specific record entry, meant to communicate the remoteness at a corresponding game
/// state.
///
/// ```none
/// [REMOTENESS_SIZE bits: Remoteness]
/// [0b0 until BUFFER_SIZE]
/// ```
///
/// The remoteness values are encoded in big-endian as unsigned integers
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
            Err(RecordViolation {
                name: RecordType::REM.into(),
                hint: format!(
                    "The record implementation operates on a buffer of {} \
                    bits, but there was an attempt to instantiate one from a \
                    buffer of {} bits.",
                    BUFFER_SIZE, len,
                ),
            })?
        } else if len < Self::minimum_bit_size() {
            Err(RecordViolation {
                name: RecordType::REM.into(),
                hint: format!(
                    "This record implementation stores remoteness values, but \
                    there was an attempt to instantiate one with from a buffer \
                    with {} bits, which is not enough to store a remoteness \
                    value (which takes {} bits).",
                    len, REMOTENESS_SIZE,
                ),
            })?
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
            Err(RecordViolation {
                name: RecordType::REM.into(),
                hint: format!(
                    "This record implementation uses {} bits to store unsigned \
                    integers representing remoteness values, but there was an \
                    attempt to store a remoteness value of {}, which requires \
                    at least {} bits to store.",
                    REMOTENESS_SIZE, value, size,
                ),
            })?
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

    // The maximum numeric remoteness value that can be expressed with exactly
    // REMOTENESS_SIZE bits in an unsigned integer.
    const MAX_REMOTENESS: Remoteness = 2_u64.pow(REMOTENESS_SIZE as u32) - 1;

    #[test]
    fn initialize_from_valid_buffer() {
        let buf = bitarr!(u8, Msb0; 0; BUFFER_SIZE);
        for i in REMOTENESS_SIZE..BUFFER_SIZE {
            assert!(RecordBuffer::from(&buf[0..i]).is_ok());
        }
    }

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
    fn set_record_attributes() {
        let mut r1 = RecordBuffer::new().unwrap();

        let good = Remoteness::MIN;
        let bad = Remoteness::MAX;

        assert!(r1.set_remoteness(good).is_ok());
        assert!(r1.set_remoteness(good).is_ok());
        assert!(r1.set_remoteness(good).is_ok());

        assert!(r1.set_remoteness(bad).is_err());
        assert!(r1.set_remoteness(bad).is_err());
        assert!(r1.set_remoteness(bad).is_err());
    }

    #[test]
    fn data_is_valid_after_round_trip() {
        let mut record = RecordBuffer::new().unwrap();
        let remoteness = 790;

        record
            .set_remoteness(remoteness)
            .unwrap();

        // Remoteness unchanged after insert and fetch
        let fetched_remoteness = record.get_remoteness();
        let actual_remoteness = remoteness;
        assert_eq!(fetched_remoteness, actual_remoteness);
    }

    #[test]
    fn extreme_data_is_valid_after_round_trip() {
        let mut record = RecordBuffer::new().unwrap();

        assert!(record
            .set_remoteness(MAX_REMOTENESS)
            .is_ok());

        assert_eq!(record.get_remoteness(), MAX_REMOTENESS);
    }
}
