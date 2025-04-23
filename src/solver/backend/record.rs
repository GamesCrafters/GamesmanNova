//! # Bitpacked Solver Record Module
//!
//! Implementation of a database record buffer for storing solver solutions for
//! different amounts of players, the remoteness value, and draw evaluation
//! associated with a particualr position.

use anyhow::Result;
use anyhow::bail;
use bitvec::BitArr;
use bitvec::bitarr;
use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::slice::BitSlice;

use crate::game::Player;
use crate::game::PlayerCount;
use crate::solver::Degree;
use crate::solver::IUtility;
use crate::solver::Remoteness;
use crate::solver::SUtility;
use crate::solver::UtilityType;
use crate::solver::error::SolverError::RecordViolation;
use crate::util;

/* MODE-INDEPENDENT */

/// The name of this record implementation, for error reporting purposes.
pub const RECORD_NAME: &str = "Bitpacked Solver Solution Record";

/// The maximum number of bits that can be used to encode a single record.
pub const BUFFER_BIT_SIZE: usize = 128;

/// The exact number of bits that are used to determine record mode.
pub const DISCRIMINANT_SIZE: usize = 1;

/* SOLUTION MODE */

/// The exact number of bits that are used to encode integer-valued utility.
pub const INTEGER_UTILITY_SIZE: usize = 4;

/// The exact number of bits that are used to encode categorical utility.
pub const SIMPLE_UTILITY_SIZE: usize = 2;

/// The exact number of bits that are used to encode a draw value.
pub const DRAW_SIZE: usize = 1;

/// The exact number of bits that are used to encode remoteness.
pub const REMOTENESS_SIZE: usize = 10;

/* EXPLORATION MODE */

/// The exact number of bits that are used to encode moves.
pub const DEGREE_SIZE: usize = 7;

/* DEFINITIONS */

/// TODO
pub enum RecordMode {
    Discovery,
    Solution {
        players: PlayerCount,
        utility: UtilityType,
        remoteness: bool,
        draw: bool,
    },
}

/// TODO
pub struct RecordBuffer {
    buf: BitArr!(for BUFFER_BIT_SIZE, in u8, Msb0),
    mode: RecordMode,
}

/* RECORD IMPLEMENTATION */

impl AsRef<[u8]> for RecordBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.buf.as_raw_slice()[..Self::byte_size(&self.mode)]
    }
}

impl RecordBuffer {
    /// Create a blank record buffer according to desired utilitty, remoteness,
    /// and draw capabilities. Fails if the desired capabilities are impossible
    /// to acommodate due to sizing constraints.
    pub fn new(mode: RecordMode) -> Result<Self> {
        let size = Self::bit_size(&mode);
        if size > BUFFER_BIT_SIZE {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "Attmept to create record exceeding buffer capacity."
                    .to_string(),
            })
        }

        match mode {
            RecordMode::Discovery => (),
            RecordMode::Solution {
                players,
                utility,
                remoteness,
                draw,
            } => {
                let max_players = Self::player_count(
                    BUFFER_BIT_SIZE,
                    remoteness,
                    utility,
                    draw,
                );

                if max_players < players {
                    bail!(RecordViolation {
                        name: RECORD_NAME,
                        hint: "Insufficient capacity in record for all player \
                            utility values."
                            .to_string(),
                    })
                }
            },
        }

        let mut buf = bitarr!(u8, Msb0; 0; BUFFER_BIT_SIZE);
        buf.set(0, matches!(mode, RecordMode::Discovery));
        Ok(Self { buf, mode })
    }

    /// Create a record buffer from a pre-existing sequence of bytes, according
    /// to desired utilitty, remoteness, and draw capabilities. Fails if the
    /// provided sequence of bits is inconsistent with the desired capabilities
    /// due to sizing constraints.
    pub fn from(bytes: &[u8], mode: RecordMode) -> Result<Self> {
        let bit_len = bytes.len() * 8;
        if bit_len > BUFFER_BIT_SIZE {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: format!(
                    "The record implementation operates on a buffer of \
                 {BUFFER_BIT_SIZE} bits, but there was an attempt to \
                 instantiate one from a buffer of {bit_len} bits.",
                ),
            })
        }

        if bytes.len() < Self::byte_size(&mode) {
            bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to instantiate a record with from \
                 a buffer that could not possibly have enough bits to store \
                 this record's capabilites or modality."
                    .to_string(),
            })
        }

        let mut buf = bitarr!(u8, Msb0; 0; BUFFER_BIT_SIZE);
        buf[0..Self::bit_size(&mode)]
            .copy_from_bitslice(BitSlice::from_slice(bytes));

        buf.set(0, matches!(mode, RecordMode::Discovery));
        Ok(Self { buf, mode })
    }

    /* GET METHODS */

    /// Obtain the degree value stored in this record buffer, failing if this
    /// record buffer does not have degree capabilities.
    pub fn get_degree(&self) -> Result<Degree> {
        match self.mode {
            RecordMode::Solution { .. } => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to get degree information from a \
                    record in solution mode, which has no degree information."
                    .to_string()
            }),
            RecordMode::Discovery => {
                let start = self.degree_bit_index()?;
                let end = start + DEGREE_SIZE;
                Ok(self.buf[start..end].load_be::<Degree>())
            },
        }
    }

    /// Obtain the remoteness value stored in this record buffer, failing if
    /// this record buffer does not have remoteness capabilities.
    pub fn get_remoteness(&self) -> Result<Remoteness> {
        match self.mode {
            RecordMode::Discovery => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to get draw information from a \
                    record in discovery mode, which has no utility information."
                    .to_string()
            }),
            RecordMode::Solution { remoteness, .. } => {
                if !remoteness {
                    bail!(RecordViolation {
                        name: RECORD_NAME,
                        hint: "Attempted to fetch remoteness from a record \
                            without remoteness capabilities."
                            .to_string()
                    })
                }

                let start = self.remoteness_bit_index()?;
                let end = start + REMOTENESS_SIZE;
                Ok(self.buf[start..end].load_be::<Remoteness>())
            },
        }
    }

    /// Obtain the draw value stored in this record buffer, failing if this
    /// record buffer does not have draw capabilities.
    pub fn get_draw(&self) -> Result<bool> {
        match self.mode {
            RecordMode::Discovery => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to get draw information from a \
                    record in discovery mode, which has no utility information."
                    .to_string()
            }),
            RecordMode::Solution { draw, .. } => {
                if !draw {
                    bail!(RecordViolation {
                        name: RECORD_NAME,
                        hint: "Attempted to fetch draw value from a record \
                            without draw capabilities."
                            .to_string()
                    })
                }

                let index = self.draw_bit_index()?;
                Ok(*self.buf.get(index).unwrap())
            },
        }
    }

    /// Obtain the integer utility value stored in this record buffer, failing
    /// if there is no such value.
    pub fn get_integer_utility(&self, player: Player) -> Result<IUtility> {
        match self.mode {
            RecordMode::Discovery => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to get integer utility from a \
                    record in discovery mode, which has no utility information."
                    .to_string()
            }),
            RecordMode::Solution { .. } => {
                let bits = self.get_utility_bits(player)?;
                Ok(bits.load_be())
            },
        }
    }

    /// Obtain the simple utility value stored in this record buffer, failing if
    /// there is no such value.
    pub fn get_simple_utility(&self, player: Player) -> Result<SUtility> {
        match self.mode {
            RecordMode::Discovery => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to get simple utility from a \
                    record in discovery mode, which has no utility information."
                    .to_string()
            }),
            RecordMode::Solution { .. } => {
                let bits = self.get_utility_bits(player)?;
                let discriminant: i8 = bits.load_be();
                Ok(discriminant.try_into()?)
            },
        }
    }

    ///

    /* SET METHODS */

    /// Set the degree associated with this record buffer. Fails if this record
    /// was created without degree storage capabilities.
    pub fn set_degree(&mut self, degree: Degree) -> Result<()> {
        match self.mode {
            RecordMode::Solution { .. } => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to set a degree value at a record \
                    in solution mode, which has no degree information."
                    .to_string()
            }),
            RecordMode::Discovery => {
                let start = self.degree_bit_index()?;
                let end = start + DEGREE_SIZE;
                self.buf[start..end].store_be(degree);
                Ok(())
            },
        }
    }

    /// Set the utility vector stored in this record buffer to a provided vector
    /// of simple-valued utilities. Fails if this record does not use simple
    /// utility values.
    pub fn set_simple_utility<const N: usize>(
        &mut self,
        v: [SUtility; N],
    ) -> Result<()> {
        match self.mode {
            RecordMode::Discovery => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to set simple utility in a record \
                    in discovery mode, which has no utility information."
                    .to_string()
            }),
            RecordMode::Solution {
                players, utility, ..
            } => {
                if N != players {
                    bail!(RecordViolation {
                        name: RECORD_NAME,
                        hint: format!(
                            "A record was instantiated with {} utility entries, \
                         and there was an attempt to use a {N}-entry utility \
                         list to update the record utility values.",
                            players,
                        ),
                    })
                }

                if !matches!(utility, UtilityType::Simple) {
                    bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an atttempt to set a vector of simple-valued \
                     utilities into a record created for another type of \
                     utility representation."
                    .into(),
            })
                }

                for player in 0..players {
                    let variant = v[player];
                    let utility = variant as i64;
                    let size = util::min_sbits(utility);
                    if size > SIMPLE_UTILITY_SIZE {
                        bail!(RecordViolation {
                            name: RECORD_NAME,
                            hint: format!(
                                "This record implementation uses \
                                {SIMPLE_UTILITY_SIZE} bits to store signed \
                                integers representing utility values, but \
                                there was an attempt to store an enum \
                                discriminant of {utility}, which requires at \
                                least {size} bits to store.",
                            ),
                        })
                    }

                    let start = self.utility_bit_index(player)?;
                    let end = start + SIMPLE_UTILITY_SIZE;
                    self.buf[start..end].store_be(utility);
                }

                Ok(())
            },
        }
    }

    /// Set the utility vector stored in this record buffer to a provided vector
    /// of integer-valued utilities. Fails if this record does not use integer
    /// utility values.
    pub fn set_integer_utility<const N: usize>(
        &mut self,
        v: [IUtility; N],
    ) -> Result<()> {
        match self.mode {
            RecordMode::Discovery => bail!(RecordViolation {
                name: RECORD_NAME,
                hint:
                    "There was an attempt to set integer utility in a record \
                    in discovery mode, which has no utility information."
                        .to_string()
            }),
            RecordMode::Solution {
                players, utility, ..
            } => {
                if N != players {
                    bail!(RecordViolation {
                        name: RECORD_NAME,
                        hint: format!(
                            "A record was instantiated with {} utility \
                            entries, and there was an attempt to use a \
                            {N}-entry utility list to update the record \
                            utility values.",
                            players,
                        ),
                    })
                }

                if !matches!(utility, UtilityType::Integer) {
                    bail!(RecordViolation {
                        name: RECORD_NAME,
                        hint: "There was an atttempt to set a vector of \
                            integer-valued utilities into a record created for \
                            another type of utility representation."
                            .into(),
                    })
                }

                for player in 0..players {
                    let utility = v[player];
                    let size = util::min_sbits(utility);
                    if size > INTEGER_UTILITY_SIZE {
                        bail!(RecordViolation {
                            name: RECORD_NAME,
                            hint: format!(
                                "This record implementation uses \
                                {INTEGER_UTILITY_SIZE} bits to store signed \
                                integers representing utility values, but \
                                there was an attempt to store a utility of \
                                {utility}, which requires at least {size} bits \
                                to store.",
                            ),
                        })
                    }

                    let start = self.utility_bit_index(player)?;
                    let end = start + INTEGER_UTILITY_SIZE;
                    self.buf[start..end].store_be(utility);
                }

                Ok(())
            },
        }
    }

    /// Set the remoteness value associatted with this record buffer. Fails if
    /// this record was created without remoteness capabilities.
    pub fn set_remoteness(&mut self, value: Remoteness) -> Result<()> {
        match self.mode {
            RecordMode::Discovery => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to set remoteness for a record in \
                    discovery mode, which has no remoteness information."
                    .to_string()
            }),
            RecordMode::Solution { remoteness, .. } => {
                let size = util::min_ubits(value);
                if !remoteness {
                    bail!(RecordViolation {
                        name: RECORD_NAME,
                        hint: "Attempted to set remoteness into a record \
                            without remoteness capabilities."
                            .to_string()
                    })
                }

                if size > REMOTENESS_SIZE {
                    bail!(RecordViolation {
                        name: RECORD_NAME,
                        hint: format!(
                            "This record implementation uses {REMOTENESS_SIZE} \
                            bits to store unsigned integers representing \
                            remoteness values, but there was an attempt to \
                            store a remoteness value of {value}, which needs \
                            at least {size} bits to store.",
                        ),
                    })
                }

                let start = self.remoteness_bit_index()?;
                let end = start + REMOTENESS_SIZE;
                self.buf[start..end].store_be(value);
                Ok(())
            },
        }
    }

    /// Set the draw value associated with this record buffer. Fails if this
    /// record was created without draw capabilities.
    pub fn set_draw(&mut self, value: bool) -> Result<()> {
        match self.mode {
            RecordMode::Discovery => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to set a draw value into a record \
                    in discovery mode, which has no draw information."
                    .to_string()
            }),
            RecordMode::Solution { draw, .. } => {
                if !draw {
                    bail!(RecordViolation {
                        name: RECORD_NAME,
                        hint:
                            "Attempted to set draw into a record without draw \
                            capabilities."
                                .to_string()
                    })
                }

                let index = self.draw_bit_index()?;
                self.buf.set(index, value);
                Ok(())
            },
        }
    }

    /* HELPERS */

    fn get_utility_bits(&self, player: Player) -> Result<&BitSlice<u8, Msb0>> {
        match self.mode {
            RecordMode::Discovery => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to find a player's utility for a \
                    record in discovery mode, which has no utility information."
                    .to_string()
            }),
            RecordMode::Solution {
                players, utility, ..
            } => {
                if player >= players {
                    bail!(RecordViolation {
                        name: RECORD_NAME,
                        hint: format!(
                            "A record was instantiated at {} utility entries, \
                        and there was an attempt to fetch utility for player \
                        {player} (0-indexed) from that record instance.",
                            players,
                        ),
                    })
                }

                let start = self.utility_bit_index(player)?;
                let end = start + utility.space();
                Ok(&self.buf[start..end])
            },
        }
    }

    /* LAYOUT FUNCTIONS */

    /// Returns true iff `bytes` has a discriminant bit set to discovery mode.
    pub fn get_discriminant(bytes: &[u8]) -> bool {
        let mut buf = bitarr!(u8, Msb0; 0; BUFFER_BIT_SIZE);
        buf[0..1].copy_from_bitslice(BitSlice::from_slice(bytes));
        buf[0]
    }

    /// Returns true iff `self` is in solution mode (not discovery mode).
    pub fn is_solution(&self) -> bool {
        self.buf[0]
    }

    #[inline(always)]
    const fn player_count(
        buffer_bit_size: usize,
        remoteness: bool,
        utility: UtilityType,
        draw: bool,
    ) -> usize {
        (buffer_bit_size
            - Self::bit_size(&RecordMode::Solution {
                players: 0,
                remoteness,
                utility,
                draw,
            }))
            / utility.space()
    }

    #[inline(always)]
    const fn bit_size(mode: &RecordMode) -> usize {
        match mode {
            RecordMode::Discovery => DISCRIMINANT_SIZE + DEGREE_SIZE,
            RecordMode::Solution {
                remoteness,
                players,
                utility,
                draw,
            } => {
                DISCRIMINANT_SIZE
                    + (*players * utility.space())
                    + if *remoteness { REMOTENESS_SIZE } else { 0 }
                    + if *draw { DRAW_SIZE } else { 0 }
            },
        }
    }

    #[inline(always)]
    const fn byte_size(mode: &RecordMode) -> usize {
        Self::bit_size(mode).div_ceil(8)
    }

    /* LAYOUT METHODS */

    fn degree_bit_index(&self) -> Result<usize> {
        match self.mode {
            RecordMode::Discovery => Ok(DISCRIMINANT_SIZE),
            RecordMode::Solution { .. } => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to find a degree index for a \
                    record in solution mode, which has no degree information."
                    .to_string()
            }),
        }
    }

    fn utility_bit_index(&self, player: Player) -> Result<usize> {
        match self.mode {
            RecordMode::Solution { utility, .. } => Ok(DISCRIMINANT_SIZE
                + DRAW_SIZE
                + REMOTENESS_SIZE
                + player * utility.space()),
            RecordMode::Discovery => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to find a utility index for a \
                    record in discovery mode, which has no utility information."
                    .to_string()
            }),
        }
    }

    fn remoteness_bit_index(&self) -> Result<usize> {
        match self.mode {
            RecordMode::Solution { .. } => Ok(DISCRIMINANT_SIZE + DRAW_SIZE),
            RecordMode::Discovery => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to find a remoteness index for a \
                    record in discovery mode, which has no such information."
                    .to_string()
            }),
        }
    }

    fn draw_bit_index(&self) -> Result<usize> {
        match self.mode {
            RecordMode::Solution { .. } => Ok(DISCRIMINANT_SIZE),
            RecordMode::Discovery => bail!(RecordViolation {
                name: RECORD_NAME,
                hint: "There was an attempt to find a draw index for a record \
                    in discovery mode, which has no such information."
                    .to_string()
            }),
        }
    }
}

impl UtilityType {
    const fn space(&self) -> usize {
        match self {
            UtilityType::Integer => INTEGER_UTILITY_SIZE,
            UtilityType::Simple => SIMPLE_UTILITY_SIZE,
        }
    }
}
