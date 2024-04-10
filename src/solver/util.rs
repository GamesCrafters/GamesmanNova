//! # Solver Utilities Module
//!
//! This module makes room for common utility routines used throughout the
//! `crate::solver` module.
//!
//! #### Authorship
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use crate::database::Schema;
use crate::solver::{record, RecordType};

/* BIT FIELDS */

/// Returns the minimum number of bits required to represent unsigned `val`.
#[inline(always)]
pub const fn min_ubits(val: u64) -> usize {
    (u64::BITS - val.leading_zeros()) as usize
}

/// Return the minimum number of bits necessary to encode `utility`, which
/// should be a signed integer in two's complement.
#[inline(always)]
pub fn min_sbits(utility: i64) -> usize {
    if utility >= 0 {
        min_ubits(utility as u64) + 1
    } else {
        min_ubits(((-utility) - 1) as u64) + 1
    }
}

/* RECORD TYPE IMPLEMENTATIONS */

impl Into<String> for RecordType {
    fn into(self) -> String {
        match self {
            RecordType::MUR(players) => {
                format!("Multi-Utility Remoteness ({} players)", players)
            },
        }
    }
}

impl TryInto<Schema> for RecordType {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Schema, Self::Error> {
        match self {
            RecordType::MUR(players) => record::mur::schema(players),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn minimum_bits_for_unsigned_integer() {
        assert_eq!(min_ubits(0), 0);
        assert_eq!(min_ubits(0b1111_1111), 8);
        assert_eq!(min_ubits(0b1001_0010), 8);
        assert_eq!(min_ubits(0b0010_1001), 6);
        assert_eq!(min_ubits(0b0000_0110), 3);
        assert_eq!(min_ubits(0b0000_0001), 1);
        assert_eq!(min_ubits(0xF000_0A00_0C00_00F5), 64);
        assert_eq!(min_ubits(0x0000_F100_DEB0_A002), 48);
        assert_eq!(min_ubits(0x0000_0000_F00B_1351), 32);
        assert_eq!(min_ubits(0x0000_0000_F020_0DE0), 32);
        assert_eq!(min_ubits(0x0000_0000_0000_FDE0), 16);
    }

    #[test]
    fn minimum_bits_for_positive_signed_integer() {
        assert_eq!(min_sbits(0x0000_8000_2222_0001), 49);
        assert_eq!(min_sbits(0x0070_DEAD_0380_7DE0), 56);
        assert_eq!(min_sbits(0x0000_0000_F00B_1351), 33);
        assert_eq!(min_sbits(0x0000_0000_0000_0700), 12);
        assert_eq!(min_sbits(0x0000_0000_0000_0001), 2);

        assert_eq!(min_sbits(-10000), 15);
        assert_eq!(min_sbits(-1000), 11);
        assert_eq!(min_sbits(-255), 9);
        assert_eq!(min_sbits(-128), 8);
        assert_eq!(min_sbits(0), 1);
    }
}
