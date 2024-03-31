//! # Solver Utilities Module
//!
//! This module makes room for common utility routines used throughout the
//! `crate::solver` module.
//!
//! #### Authorship
//!
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use crate::solver::{record, RecordType};
use crate::{database::Schema, model::Utility};

/* BIT FIELDS */

/// Returns the minimum number of bits required to represent unsigned `val`.
#[inline(always)]
pub const fn min_ubits(val: u64) -> usize {
    let mut x = 0;
    while (val >> x != 0b1) && (x != u64::BITS) {
        x += 1;
    }
    (u64::BITS - x) as usize
}

/// Return the minimum number of bits necessary to encode `utility`.
#[inline(always)]
pub const fn min_sbits(utility: Utility) -> usize {
    let mut res = 0;
    let mut inter = utility.abs();
    while inter != 0 {
        inter >>= 1;
        res += 1;
    }
    res + 1
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
