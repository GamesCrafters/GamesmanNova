//! # General Utilities Module
//!
//! This module makes room for verbose or repeated routines used in the
//! top-level module of this crate.

use std::collections::HashSet;
use std::hash::Hash;

/* BIT FIELDS */

/// Returns the minimum number of bits required to represent unsigned `val`.
#[inline(always)]
pub const fn min_ubits(val: u64) -> usize {
    (u64::BITS - val.leading_zeros()) as usize
}

/* MISC */

/// Returns the first duplicate found in `vec`.
pub fn first_duplicate<T: Eq + Hash + Clone>(vec: &[T]) -> Option<T> {
    let mut seen = HashSet::new();
    for item in vec {
        if !seen.insert(item) {
            return Some(item.clone());
        }
    }
    None
}

/* DECLARATIVE MACROS */

/// Syntax sugar. Allows for a declarative way of expressing extensive game
/// state nodes.
///
/// # Example
///
/// ```ignore
/// // A medial node where it is Player 5's turn.
/// let n1 = node!(5);
///
/// // A terminal node with a 5-entry utility vector, on player 2's turn.
/// let n2 = node![2; -1, -4, 5, 0, 3];
///
/// // A terminal node with a single utility entry, on player 1's turn.
/// let n3 = node![1; 4];
/// ```
#[macro_export]
macro_rules! node {
    ($val:expr) => {
        Node::Medial($val)
    };
    ($player:expr; $($u:expr),+ $(,)?) => {
        Node::Terminal($player, vec![$($u),*])
    };
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
}
