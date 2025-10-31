//! # Solver Record Models
//!
//! TODO

use bitvec::BitArr;
use bitvec::order::Msb0;

use crate::model::game::PlayerCount;
use crate::model::game::UtilityType;

/* CONSTANTS */

/// The maximum number of bits that can be used to encode a single record.
pub const BUFFER_BIT_SIZE: usize = 128;

/// The exact number of bits that are used to determine record mode.
pub const DISCRIMINANT_SIZE: usize = 1;

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

/* ENUMERATIONS */

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
    pub buf: BitArr!(for BUFFER_BIT_SIZE, in u8, Msb0),
    pub mode: RecordMode,
}
