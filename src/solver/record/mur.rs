//! # Multi-Utility Remoteness (MUR) Record Module
//!
//! Implementation of a database record buffer for storing the utility
//! information of different amounts of players and the remoteness value
//! associated with a particular game state.
//!
//! #### Authorship
//!
//! - Max Fierro, 3/30/2024 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};
use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::slice::BitSlice;
use bitvec::{bitarr, BitArr};

use crate::database::{Attribute, Datatype, Record, Schema, SchemaBuilder};
use crate::model::{PlayerCount, Remoteness, Turn, Utility};
use crate::solver::error::SolverError::RecordViolation;
use crate::solver::util;
use crate::solver::RecordType;

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
            name: RecordType::MUR(players).into(),
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
                name: RecordType::MUR(players).into(),
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
                name: RecordType::MUR(0).into(),
                hint: format!(
                    "The record implementation operates on a buffer of {} \
                    bits, but there was an attempt to instantiate one from a \
                    buffer of {} bits.",
                    BUFFER_SIZE, len,
                ),
            })?
        } else if len < Self::minimum_bit_size() {
            Err(RecordViolation {
                name: RecordType::MUR(0).into(),
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
    pub fn get_utility(&self, player: Turn) -> Result<Utility> {
        if player >= self.players {
            Err(RecordViolation {
                name: RecordType::MUR(self.players).into(),
                hint: format!(
                    "A record was instantiated with {} utility entries, and \
                    there was an attempt to fetch the utility of player {} \
                    (0-indexed) from that record instance.",
                    self.players, player,
                ),
            })?
        } else {
            let start = Self::utility_index(self.players);
            let end = start + UTILITY_SIZE;
            Ok(self.buf[start..end].load_be::<Utility>())
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
        v: [Utility; N],
    ) -> Result<()> {
        if N >= self.players {
            Err(RecordViolation {
                name: RecordType::MUR(self.players).into(),
                hint: format!(
                    "A record was instantiated with {} utility entries, and \
                    there was an attempt to set the utility of player {} \
                    (0-indexed) from that record instance.",
                    self.players, N,
                ),
            })?
        } else {
            let player = 0;
            while player < self.players {
                let utility = v[player];
                let size = util::min_sbits(utility);
                if size > UTILITY_SIZE {
                    Err(RecordViolation {
                        name: RecordType::MUR(self.players).into(),
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
                name: RecordType::MUR(self.players).into(),
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
        players * UTILITY_SIZE + REMOTENESS_SIZE
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
    const fn utility_index(player: Turn) -> usize {
        player * UTILITY_SIZE
    }

    /// Return the maximum number of utility entries supported by a dense record
    /// (one that maximizes bit usage) with `length`. Ignores unused bits.
    #[inline(always)]
    const fn player_count(length: usize) -> usize {
        length - REMOTENESS_SIZE / UTILITY_SIZE
    }
}
