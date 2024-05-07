//! # Multi-Utility Remoteness (MUR) Record Module
//!
//! Implementation of a database record buffer for storing the utility
//! information of different amounts of players and the remoteness value
//! associated with a particular game state.
//!
//! #### Authorship
//! - Max Fierro, 3/30/2024 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};
use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::slice::BitSlice;
use bitvec::{bitarr, BitArr};

use crate::database::{Attribute, Datatype, Record, Schema, SchemaBuilder};
use crate::model::game::{Player, PlayerCount};
use crate::model::solver::{IUtility, Remoteness};
use crate::solver::error::SolverError::RecordViolation;
use crate::solver::RecordType;
use crate::util;

/* CONSTANTS */

/// The exact number of bits that are used to encode remoteness.
pub const REMOTENESS_SIZE: usize = 16;

/// The maximum number of bits that can be used to encode a single record.
pub const BUFFER_SIZE: usize = 128;

/// The exact number of bits that are used to encode utility for one player.
pub const UTILITY_SIZE: usize = 8;

/* SCHEMA GENERATOR */

/// Return the database table schema associated with a record instance with
/// a specific number of `players` under this record implementation.
pub fn schema(players: PlayerCount) -> Result<Schema> {
    if RecordBuffer::bit_size(players) > BUFFER_SIZE {
        Err(RecordViolation {
            name: RecordType::MUR(players).to_string(),
            hint: format!(
                "This record can only hold utility values for up to {} \
                players, but there was an attempt to create a schema that \
                would represent one holding {} players.",
                RecordBuffer::player_count(BUFFER_SIZE),
                players
            ),
        })?
    } else {
        let mut schema = SchemaBuilder::new().of(RecordType::MUR(players));

        for i in 0..players {
            let name = &format!("P{} utility", i);
            let data = Datatype::SINT;
            let size = UTILITY_SIZE;
            schema = schema
                .add(Attribute::new(name, data, size))
                .context(
                    "Failed to add utility attribute to database schema.",
                )?;
        }

        let name = "State remoteness";
        let data = Datatype::UINT;
        let size = REMOTENESS_SIZE;
        schema = schema
            .add(Attribute::new(name, data, size))
            .context(
                "Failed to add remoteness attribute to database schema.",
            )?;

        Ok(schema.build())
    }
}

/* RECORD IMPLEMENTATION */

/// Solver-specific record entry, meant to communicate the remoteness and each
/// player's utility at a corresponding game state. The layout is as follows:
///
/// ```none
/// [UTILITY_SIZE bits: P0 utility]
/// ...
/// [UTILITY_SIZE bits: P(N-1) utility]
/// [REMOTENESS_SIZE bits: Remoteness]
/// [0b0 until BUFFER_SIZE]
/// ```
///
/// The number of players `N` is limited by `BUFFER_SIZE`, because a statically
/// sized buffer is used for intermediary storage. The utility and remoteness
/// values are encoded in big-endian, with utility being a signed two's
/// complement integer and remoteness an unsigned integer.
pub struct RecordBuffer {
    buf: BitArr!(for BUFFER_SIZE, in u8, Msb0),
    players: PlayerCount,
}

impl Record for RecordBuffer {
    #[inline(always)]
    fn raw(&self) -> &BitSlice<u8, Msb0> {
        &self.buf[..Self::bit_size(self.players)]
    }
}

impl RecordBuffer {
    /// Returns a new instance of a bit-packed record buffer that is able to
    /// store utility values for `players`. Fails if `players` is too high for
    /// the underlying buffer's capacity.
    #[inline(always)]
    pub fn new(players: PlayerCount) -> Result<Self> {
        if Self::bit_size(players) > BUFFER_SIZE {
            Err(RecordViolation {
                name: RecordType::MUR(players).to_string(),
                hint: format!(
                    "The record can only hold utility values for up to {} \
                    players, but there was an attempt to instantiate one for \
                    {} players.",
                    Self::player_count(BUFFER_SIZE),
                    players
                ),
            })?
        } else {
            Ok(Self {
                buf: bitarr!(u8, Msb0; 0; BUFFER_SIZE),
                players,
            })
        }
    }

    /// Return a new instance with `bits` as the underlying buffer. Fails in the
    /// event that the size of `bits` is incoherent with the record.
    #[inline(always)]
    pub fn from(bits: &BitSlice<u8, Msb0>) -> Result<Self> {
        let len = bits.len();
        if len > BUFFER_SIZE {
            Err(RecordViolation {
                name: RecordType::MUR(0).to_string(),
                hint: format!(
                    "The record implementation operates on a buffer of {} \
                    bits, but there was an attempt to instantiate one from a \
                    buffer of {} bits.",
                    BUFFER_SIZE, len,
                ),
            })?
        } else if len < Self::minimum_bit_size() {
            Err(RecordViolation {
                name: RecordType::MUR(0).to_string(),
                hint: format!(
                    "This record implementation stores utility values, but \
                    there was an attempt to instantiate one with from a buffer \
                    with {} bits, which is not enough to store a remoteness \
                    value (which takes {} bits).",
                    len, REMOTENESS_SIZE,
                ),
            })?
        } else {
            let players = Self::player_count(len);
            let mut buf = bitarr!(u8, Msb0; 0; BUFFER_SIZE);
            buf[..len].copy_from_bitslice(bits);
            Ok(Self { players, buf })
        }
    }

    /* GET METHODS */

    /// Parse and return the utility value corresponding to `player`. Fails if
    /// the `player` index passed in is incoherent with player count.
    #[inline(always)]
    pub fn get_utility(&self, player: Player) -> Result<IUtility> {
        if player >= self.players {
            Err(RecordViolation {
                name: RecordType::MUR(self.players).to_string(),
                hint: format!(
                    "A record was instantiated with {} utility entries, and \
                    there was an attempt to fetch the utility of player {} \
                    (0-indexed) from that record instance.",
                    self.players, player,
                ),
            })?
        } else {
            let start = Self::utility_index(player);
            let end = start + UTILITY_SIZE;
            Ok(self.buf[start..end].load_be::<IUtility>())
        }
    }

    /// Parse and return the remoteness value in the record encoding. Failure
    /// here indicates corrupted state.
    #[inline(always)]
    pub fn get_remoteness(&self) -> Remoteness {
        let start = Self::remoteness_index(self.players);
        let end = start + REMOTENESS_SIZE;
        self.buf[start..end].load_be::<Remoteness>()
    }

    /* SET METHODS */

    /// Set this entry to have the utility values in `v` for each player. Fails
    /// if any of the utility values are too high to fit in the space dedicated
    /// for each player's utility, or if there is a mismatch between player
    /// count and the number of utility values passed in.
    #[inline(always)]
    pub fn set_utility<const N: usize>(
        &mut self,
        v: [IUtility; N],
    ) -> Result<()> {
        if N != self.players {
            Err(RecordViolation {
                name: RecordType::MUR(self.players).to_string(),
                hint: format!(
                    "A record was instantiated with {} utility entries, and \
                    there was an attempt to use a {}-entry utility list to \
                    update the record utility values.",
                    self.players, N,
                ),
            })?
        } else {
            for player in 0..self.players {
                let utility = v[player];
                let size = util::min_sbits(utility);
                if size > UTILITY_SIZE {
                    Err(RecordViolation {
                        name: RecordType::MUR(self.players).to_string(),
                        hint: format!(
                            "This record implementation uses {} bits to store \
                            signed integers representing utility values, but \
                            there was an attempt to store a utility of {}, \
                            which requires at least {} bits to store.",
                            UTILITY_SIZE, utility, size,
                        ),
                    })?
                }

                let start = Self::utility_index(player);
                let end = start + UTILITY_SIZE;
                self.buf[start..end].store_be(utility);
            }
            Ok(())
        }
    }

    /// Set this entry to have `value` remoteness. Fails if `value` is too high
    /// to fit in the space dedicated for remoteness within the record.
    #[inline(always)]
    pub fn set_remoteness(&mut self, value: Remoteness) -> Result<()> {
        let size = util::min_ubits(value);
        if size > REMOTENESS_SIZE {
            Err(RecordViolation {
                name: RecordType::MUR(self.players).to_string(),
                hint: format!(
                    "This record implementation uses {} bits to store unsigned \
                    integers representing remoteness values, but there was an \
                    attempt to store a remoteness value of {}, which requires \
                    at least {} bits to store.",
                    REMOTENESS_SIZE, value, size,
                ),
            })?
        } else {
            let start = Self::remoteness_index(self.players);
            let end = start + REMOTENESS_SIZE;
            self.buf[start..end].store_be(value);
            Ok(())
        }
    }

    /* LAYOUT HELPER METHODS */

    /// Return the number of bits that would be needed to store a record
    /// containing utility information for `players` as well as remoteness.
    #[inline(always)]
    const fn bit_size(players: usize) -> usize {
        (players * UTILITY_SIZE) + REMOTENESS_SIZE
    }

    /// Return the minimum number of bits needed for a valid record buffer.
    #[inline(always)]
    const fn minimum_bit_size() -> usize {
        REMOTENESS_SIZE
    }

    /// Return the bit index of the remoteness entry start in the record buffer.
    #[inline(always)]
    const fn remoteness_index(players: usize) -> usize {
        players * UTILITY_SIZE
    }

    /// Return the bit index of the 'i'th player's utility entry start.
    #[inline(always)]
    const fn utility_index(player: Player) -> usize {
        player * UTILITY_SIZE
    }

    /// Return the maximum number of utility entries supported by a dense record
    /// (one that maximizes bit usage) with `length`. Ignores unused bits.
    #[inline(always)]
    const fn player_count(length: usize) -> usize {
        (length - REMOTENESS_SIZE) / UTILITY_SIZE
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // The maximum and minimum numeric values that can be represented with
    // exactly UTILITY_SIZE bits in two's complement.
    //
    // Example if UTILITY_SIZE is 8:
    //
    // * `MAX_UTILITY = 0b01111111 = 127 = 2^(8 - 1) - 1`
    // * `MIN_UTILITY = 0b10000000 = -128 =  -127 - 1`
    //
    // Useful: https://www.omnicalculator.com/math/twos-complement
    const MAX_UTILITY: IUtility = 2_i64.pow(UTILITY_SIZE as u32 - 1) - 1;
    const MIN_UTILITY: IUtility = (-MAX_UTILITY) - 1;

    // The maximum numeric remoteness value that can be expressed with exactly
    // REMOTENESS_SIZE bits in an unsigned integer.
    const MAX_REMOTENESS: Remoteness = 2_u64.pow(REMOTENESS_SIZE as u32) - 1;

    #[test]
    fn initialize_with_valid_player_count() {
        for i in 0..=RecordBuffer::player_count(BUFFER_SIZE) {
            assert!(RecordBuffer::new(i).is_ok())
        }
    }

    #[test]
    fn initialize_with_invalid_player_count() {
        let max = RecordBuffer::player_count(BUFFER_SIZE);

        assert!(RecordBuffer::new(max + 1).is_err());
        assert!(RecordBuffer::new(max + 10).is_err());
        assert!(RecordBuffer::new(max + 100).is_err());
    }

    #[test]
    fn initialize_from_valid_buffer() {
        let buf = bitarr!(u8, Msb0; 0; BUFFER_SIZE);
        for i in REMOTENESS_SIZE..BUFFER_SIZE {
            assert!(RecordBuffer::from(&buf[0..i]).is_ok());
        }
    }

    #[test]
    fn initialize_from_invalid_buffer() {
        let buf1 = bitarr!(u8, Msb0; 0; BUFFER_SIZE + 1);
        let buf2 = bitarr!(u8, Msb0; 0; BUFFER_SIZE + 10);
        let buf3 = bitarr!(u8, Msb0; 0; BUFFER_SIZE + 100);

        assert!(RecordBuffer::from(&buf1).is_err());
        assert!(RecordBuffer::from(&buf2).is_err());
        assert!(RecordBuffer::from(&buf3).is_err());
    }

    #[test]
    fn set_record_attributes() {
        let mut r1 = RecordBuffer::new(7).unwrap();
        let mut r2 = RecordBuffer::new(4).unwrap();
        let mut r3 = RecordBuffer::new(0).unwrap();

        let v1 = [-24; 7];
        let v2 = [113; 4];
        let v3: [IUtility; 0] = [];

        let v4 = [IUtility::MAX; 7];
        let v5 = [-IUtility::MAX; 4];
        let v6 = [1];

        let good = Remoteness::MIN;
        let bad = Remoteness::MAX;

        assert!(r1.set_utility(v1).is_ok());
        assert!(r2.set_utility(v2).is_ok());
        assert!(r3.set_utility(v3).is_ok());

        assert!(r1.set_utility(v4).is_err());
        assert!(r2.set_utility(v5).is_err());
        assert!(r3.set_utility(v6).is_err());

        assert!(r1.set_remoteness(good).is_ok());
        assert!(r2.set_remoteness(good).is_ok());
        assert!(r3.set_remoteness(good).is_ok());

        assert!(r1.set_remoteness(bad).is_err());
        assert!(r2.set_remoteness(bad).is_err());
        assert!(r3.set_remoteness(bad).is_err());
    }

    #[test]
    fn data_is_valid_after_round_trip() -> Result<()> {
        let mut record = RecordBuffer::new(5).unwrap();
        let payoffs = [10, -2, -8, 100, 0];
        let remoteness = 790;

        record
            .set_utility(payoffs)
            .unwrap();

        record
            .set_remoteness(remoteness)
            .unwrap();

        for (i, actual) in payoffs.iter().enumerate() {
            assert_eq!(record.get_utility(i)?, *actual);
        }

        assert_eq!(record.get_remoteness(), remoteness);
        assert!(record.get_utility(5).is_err());
        assert!(record.get_utility(10).is_err());

        Ok(())
    }

    #[test]
    fn extreme_data_is_valid_after_round_trip() -> Result<()> {
        let mut record = RecordBuffer::new(6).unwrap();

        let good = [
            MAX_UTILITY,
            MIN_UTILITY,
            MAX_UTILITY - 1,
            MIN_UTILITY + 1,
            MAX_UTILITY - 16,
            MIN_UTILITY + 16,
        ];

        let bad = [
            MAX_UTILITY + 16,
            MAX_UTILITY + 2,
            MAX_UTILITY + 1,
            MIN_UTILITY - 16,
            MIN_UTILITY - 2,
            MIN_UTILITY - 1,
        ];

        record.set_utility(good)?;
        record.set_remoteness(MAX_REMOTENESS)?;
        for (i, actual) in good.iter().enumerate() {
            assert_eq!(record.get_utility(i)?, *actual);
        }

        assert_eq!(record.get_remoteness(), MAX_REMOTENESS);
        assert!(record.set_utility(bad).is_err());

        Ok(())
    }
}
