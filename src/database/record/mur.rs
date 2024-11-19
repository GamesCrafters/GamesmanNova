//! # Multi-Utility Remoteness (MUR) Record Module
//!
//! Implementation of a database record buffer for storing the utility
//! information of different amounts of players, the remoteness value, and draw
//! evaluation associated with a particualr position.

use anyhow::{bail, Context, Result};
use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::slice::BitSlice;
use bitvec::{bitarr, BitArr};

use crate::database::error::DatabaseError::RecordViolation;
use crate::database::{Attribute, Datatype, Record, Schema, SchemaBuilder};
use crate::game::model::{Player, PlayerCount};
use crate::solver::model::{IUtility, RUtility, Remoteness, SUtility};
use crate::solver::UtilityType;
use crate::util;

/* CONSTANTS */

/// The name of this record implementation, for error reporting purposes.
pub const RECORD_NAME: &str = "Multi-Utility Remoteness Draw Record";

/// The maximum number of bits that can be used to encode a single record.
pub const BUFFER_SIZE: usize = 128;

/// The exact number of bits that are used to encode a draw value.
pub const DRAW_SIZE: usize = 1;

/// The exact number of bits that are used to encode remoteness.
pub const REMOTENESS_SIZE: usize = 13;

/// The exact number of bits that are used to encode real-valued utility.
pub const FLOAT_UTILITTY_SIZE: usize = 64;

/// The exact number of bits that are used to encode integer-valued utility.
pub const INTEGER_UTILITY_SIZE: usize = 8;

/// The exact number of bits that are used to encode categorical utility.
pub const SIMPLE_UTILITY_SIZE: usize = 2;

/* DEFINITIONS */

/// An implementation of a general-purpose record with the ability to store
/// utility values for a variable number of players, as well as the option to
/// store remoteness and draw values.
///
/// # Layout
///
/// This buffer automatically resizes according to its desired capabilities.
/// Its layout is as follows:
///
/// ```
/// [p0 utility]...[pN utility][remoteness][draw]
/// ```
///
/// It is possible for any of these values to be missing. In this case, there
/// will not be a blank space where the value would have been -- instead, the
/// next value is pushed to the leftmost position. This ensures that a
/// minimally-sized representation of the record can be provided.
pub struct RecordBuffer {
    buf: BitArr!(for BUFFER_SIZE, in u8, Msb0),
    players: PlayerCount,
    utility: UtilityType,
    remoteness: bool,
    draw: bool,
}

/* SCHEMA GENERATOR */

/// Return a database table schema for a record storing utility values of a
/// specified `utility` type for a provided number of `players`, with the option
/// of storing `remoteness` and `draw` values.
pub fn schema(
    players: PlayerCount,
    utility: UtilityType,
    remoteness: bool,
    draw: bool,
) -> Result<Schema> {
    let bit_size = RecordBuffer::bit_size(players, utility, remoteness, draw);
    let max_players =
        RecordBuffer::player_count(BUFFER_SIZE, utility, remoteness, draw);

    if bit_size > BUFFER_SIZE {
        bail!(RecordViolation {
            name: RECORD_NAME,
            hint: format!(
                "This record can hold utility values for up to {max_players} \
                players, but there was an attempt to create a schema that \
                would represent one holding {players} players.",
            ),
        })
    }

    let (data, size) = match utility {
        UtilityType::Integer => (Datatype::SINT, INTEGER_UTILITY_SIZE),
        UtilityType::Simple => (Datatype::ENUM, SIMPLE_UTILITY_SIZE),
        UtilityType::Real => (Datatype::SPFP, FLOAT_UTILITTY_SIZE),
    };

    let mut schema = SchemaBuilder::new();
    for i in 0..players {
        let name = &format!("p{i}_utility");
        schema = schema
            .add(Attribute::new(name, data, size))
            .context("Failed to add utility attribute to database schema.")?;
    }

    if remoteness {
        schema = schema.add(Attribute::new(
            "remoteness",
            Datatype::UINT,
            REMOTENESS_SIZE,
        ))?;
    }

    if draw {
        schema =
            schema.add(Attribute::new("draw", Datatype::BOOL, DRAW_SIZE))?;
    }

    Ok(schema.build())
}

/* RECORD IMPLEMENTATION */

impl Record for RecordBuffer {
    #[inline(always)]
    fn raw(&self) -> &BitSlice<u8, Msb0> {
        &self.buf[..Self::bit_size(
            self.players,
            self.utility,
            self.remoteness,
            self.draw,
        )]
    }
}

impl RecordBuffer {
    /// Create a blank record buffer according to desired utilitty, remoteness,
    /// and draw capabilities. Fails if the desired capabilities are impossible
    /// to acommodate due to sizing constraints.
    pub fn new(
        players: PlayerCount,
        utility: UtilityType,
        remoteness: bool,
        draw: bool,
    ) -> Result<Self> {
        let size = Self::bit_size(players, utility, remoteness, draw);
        let max_players =
            Self::player_count(BUFFER_SIZE, utility, remoteness, draw);

        if size > BUFFER_SIZE {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "The record can hold utilities for up to {max_players} \
                    players, but there was an attempt to instantiate one for \
                    {players} players.",
                ),
            })
        }
        Ok(Self {
            buf: bitarr!(u8, Msb0; 0; BUFFER_SIZE),
            remoteness,
            players,
            utility,
            draw,
        })
    }

    /// Create a record buffer from a pre-existing sequence of bits, according
    /// to desired utilitty, remoteness, and draw capabilities. Fails if the
    /// provided sequence of bits is inconsistent with the desired capabilities
    /// due to sizing constraints.
    pub fn from(
        bits: &BitSlice<u8, Msb0>,
        utility: UtilityType,
        remoteness: bool,
        draw: bool,
    ) -> Result<Self> {
        let len = bits.len();
        if len > BUFFER_SIZE {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "The record implementation operates on a buffer of \
                    {BUFFER_SIZE} bits, but there was an attempt to \
                    instantiate one from a buffer of {len} bits.",
                ),
            })
        }

        if len < Self::minimum_bit_size(remoteness, draw) {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "There was an attempt to instantiate this record with from \
                    a buffer that could not possibly have enough bits to store \
                    this record's capabilities (remotness: {remoteness}, draw: \
                    {draw}).",
                ),
            })
        }

        let players = Self::player_count(len, utility, remoteness, draw);
        let mut buf = bitarr!(u8, Msb0; 0; BUFFER_SIZE);
        buf[0..len].copy_from_bitslice(bits);
        Ok(Self {
            remoteness,
            players,
            utility,
            draw,
            buf,
        })
    }

    /* GET METHODS */

    /// Obtain the remoteness value stored in this record buffer, failing if
    /// this record buffer does not have remoteness capabilities.
    pub fn get_remoteness(&self) -> Result<Remoteness> {
        if !self.remoteness {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "Attempted to fetch remoteness from a record without \
                    remoteness capabilities."
                    .to_string()
            })
        }

        let start = self.remoteness_index();
        let end = start + REMOTENESS_SIZE;
        Ok(self.buf[start..end].load_be::<Remoteness>())
    }

    /// Obtain the draw value stored in this record buffer, failing if this
    /// record buffer does not have draw capabilities.
    pub fn get_draw(&self) -> Result<bool> {
        if !self.draw {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "Attempted to fetch draw value from a record without \
                    draw capabilities."
                    .to_string()
            })
        }

        let index = self.draw_index();
        Ok(*self.buf.get(index).unwrap())
    }

    /// Obtain the integer utility value stored in this record buffer, failing
    /// if there is no such value.
    pub fn get_integer_utility(&self, player: Player) -> Result<IUtility> {
        let bits = self.get_utility_bits(player)?;
        Ok(bits.load_be())
    }

    /// Obtain the simple utility value stored in this record buffer, failing if
    /// there is no such value.
    pub fn get_simple_utility(&self, player: Player) -> Result<SUtility> {
        let bits = self.get_utility_bits(player)?;
        let discriminant: i8 = bits.load_be();
        Ok(discriminant.try_into()?)
    }

    /// Obtain the real utility value stored in this record buffer, failing if
    /// there is no such value.
    pub fn get_real_utility(&self, player: Player) -> Result<RUtility> {
        todo!()
    }

    /* SET METHODS */

    /// Set the utility vector stored in this record buffer to a provided vector
    /// of simple-valued utilities. Fails if this record does not use simple
    /// utility values.
    pub fn set_simple_utility<const N: usize>(
        &mut self,
        v: [SUtility; N],
    ) -> Result<()> {
        if N != self.players {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "A record was instantiated with {} utility entries, \
                        and there was an attempt to use a {N}-entry utility \
                        list to update the record utility values.",
                    self.players,
                ),
            })
        }

        if !matches!(self.utility, UtilityType::Simple) {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "There was an atttempt to set a vector of simple-valued \
                    utilities into a record created for another type of \
                    utility representation.",
                ),
            })
        }

        for player in 0..self.players {
            let variant = v[player];
            let utility = variant as i64;
            let size = util::min_sbits(utility);
            if size > SIMPLE_UTILITY_SIZE {
                bail!(RecordViolation {
                    name: RECORD_NAME,
                    hint: format!(
                        "This record implementation uses {SIMPLE_UTILITY_SIZE} \
                        bits to store signed integers representing utility \
                        values, but there was an attempt to store an enum \
                        discriminant of {utility}, which requires at least \
                        {size} bits to store.",
                    ),
                })
            }

            let start = self.utility_index(player);
            let end = start + SIMPLE_UTILITY_SIZE;
            self.buf[start..end].store_be(utility);
        }

        Ok(())
    }

    /// Set the utility vector stored in this record buffer to a provided vector
    /// of integer-valued utilities. Fails if this record does not use integer
    /// utility values.
    pub fn set_integer_utility<const N: usize>(
        &mut self,
        v: [IUtility; N],
    ) -> Result<()> {
        if N != self.players {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "A record was instantiated with {} utility entries, and \
                    there was an attempt to use a {N}-entry utility list to \
                    update the record utility values.",
                    self.players,
                ),
            })
        }

        if !matches!(self.utility, UtilityType::Integer) {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "There was an atttempt to set a vector of integer-valued \
                    utilities into a record created for another type of \
                    utility representation.",
                ),
            })
        }

        for player in 0..self.players {
            let utility = v[player];
            let size = util::min_sbits(utility);
            if size > INTEGER_UTILITY_SIZE {
                bail!(RecordViolation {
                    name: RECORD_NAME,
                    hint: format!(
                        "This record implementation uses \
                        {INTEGER_UTILITY_SIZE} bits to store signed integers \
                        representing utility values, but there was an attempt \
                        to store a utility of {utility}, which requires at \
                        least {size} bits to store.",
                    ),
                })
            }

            let start = self.utility_index(player);
            let end = start + INTEGER_UTILITY_SIZE;
            self.buf[start..end].store_be(utility);
        }

        Ok(())
    }

    /// Set the utility vector stored in this record buffer to a provided vector
    /// of real-valued utilities. Fails if this record does not use real utility
    /// values.
    pub fn set_real_utility<const N: usize>(
        &mut self,
        v: [RUtility; N],
    ) -> Result<()> {
        todo!()
    }

    /// Set the remoteness value associatted with this record buffer. Fails if
    /// this record was created without remoteness capabilities.
    pub fn set_remoteness(&mut self, value: Remoteness) -> Result<()> {
        let size = util::min_ubits(value);
        if !self.remoteness {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "Attempted to set remoteness into a record without \
                    remoteness capabilities."
                    .to_string()
            })
        }

        if size > REMOTENESS_SIZE {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "This record implementation uses {REMOTENESS_SIZE} bits to \
                    store unsigned integers representing remoteness values, \
                    but there was an attempt to store a remoteness value of \
                    {value}, which requires at least {size} bits to store.",
                ),
            })
        }

        let start = self.remoteness_index();
        let end = start + REMOTENESS_SIZE;
        self.buf[start..end].store_be(value);
        Ok(())
    }

    /// Set the draw value associated with this record buffer. Fails if this
    /// record was created without draw capabilities.
    pub fn set_draw(&mut self, value: bool) -> Result<()> {
        if !self.draw {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "Attempted to set draw into a record without draw \
                    capabilities."
                    .to_string()
            })
        }

        let index = self.draw_index();
        self.buf.set(index, value);
        Ok(())
    }

    /* HELPERS */

    fn get_utility_bits(&self, player: Player) -> Result<&BitSlice<u8, Msb0>> {
        if player >= self.players {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "A record was instantiated with {} utility entries, and \
                    there was an attempt to fetch the utility of player \
                    {player} (0-indexed) from that record instance.",
                    self.players,
                ),
            })
        }

        let start = self.utility_index(player);
        let end = start + self.utility.space();
        Ok(&self.buf[start..end])
    }

    /* LAYOUT FUNCTIONS */

    #[inline(always)]
    const fn player_count(
        length: usize,
        utility: UtilityType,
        remoteness: bool,
        draw: bool,
    ) -> usize {
        (length
            - if remoteness { REMOTENESS_SIZE } else { 0 }
            - if draw { DRAW_SIZE } else { 0 })
            / utility.space()
    }

    #[inline(always)]
    const fn bit_size(
        players: usize,
        utility: UtilityType,
        remoteness: bool,
        draw: bool,
    ) -> usize {
        (players * utility.space())
            + if remoteness { REMOTENESS_SIZE } else { 0 }
            + if draw { DRAW_SIZE } else { 0 }
    }

    #[inline(always)]
    const fn minimum_bit_size(remoteness: bool, draw: bool) -> usize {
        0 + if remoteness { REMOTENESS_SIZE } else { 0 }
            + if draw { DRAW_SIZE } else { 0 }
    }

    /* LAYOUT METHODS */

    #[inline(always)]
    const fn utility_index(&self, player: Player) -> usize {
        player * self.utility.space()
    }

    #[inline(always)]
    const fn remoteness_index(&self) -> usize {
        self.players * self.utility.space()
    }

    #[inline(always)]
    const fn draw_index(&self) -> usize {
        self.remoteness_index()
            + if self.remoteness { REMOTENESS_SIZE } else { 0 }
    }
}

impl UtilityType {
    const fn space(&self) -> usize {
        match self {
            UtilityType::Integer => INTEGER_UTILITY_SIZE,
            UtilityType::Simple => SIMPLE_UTILITY_SIZE,
            UtilityType::Real => FLOAT_UTILITTY_SIZE,
        }
    }
}

#[cfg(test)]
mod test {
    // TODO
}
