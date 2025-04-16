//! # General Utilities Module
//!
//! This module makes room for verbose or repeated routines used in the
//! top-level module of this crate.

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use sqlx::SqlitePool;

use std::collections::HashSet;
use std::env;
use std::hash::Hash;

use crate::game;

/* BIT FIELDS */

/// Returns the minimum number of bits required to represent unsigned `val`.
#[inline(always)]
pub const fn min_ubits(val: u64) -> usize {
    (u64::BITS - val.leading_zeros()) as usize
}

/* DATABASE */

/// Returns handle to the global game solution database.
pub fn game_db() -> Result<SqlitePool> {
    let db = game::DB
        .get()
        .ok_or(anyhow!("Failed to access database singleton."))?;

    Ok(db.clone())
}

/// Parses environment variables and establishes an SQLite connection to the
/// global game solution database.
pub async fn prepare() -> Result<()> {
    dotenv::dotenv()
        .context("Failed to parse settings in environment (.env) file.")?;

    let db_addr = format!(
        "sqlite://{}",
        env::var("DATABASE")
            .context("DATABASE environment variable not set.")?
    );

    let db_pool = SqlitePool::connect(&db_addr)
        .await
        .context("Failed to initialize SQLite connection.")?;

    let _ = game::DB.set(db_pool);
    Ok(())
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
/// ```no_run
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
