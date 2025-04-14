//! # General Utilities Module
//!
//! This module makes room for verbose or repeated routines used in the
//! top-level module of this crate.

use anyhow::{Context, Result, anyhow};
use sqlx::SqlitePool;

use std::env;

use crate::game;

/* BIT FIELDS */

/// Returns the minimum number of bits required to represent unsigned `val`.
#[inline(always)]
pub const fn min_ubits(val: u64) -> usize {
    (u64::BITS - val.leading_zeros()) as usize
}

/* DATABASE */

/// TODO
pub fn game_db() -> Result<SqlitePool> {
    let db = game::DB
        .get()
        .ok_or(anyhow!("Failed to access database singleton."))?;

    Ok(db.clone())
}

/// TODO
pub async fn prepare() -> Result<()> {
    dotenv::dotenv().context("Failed to parse environment (.env) file.")?;

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

/* DECLARATIVE MACROS */

/// Syntax sugar. Implements multiple traits for a single concrete type. The
/// traits implemented must be marker traits; in other words, they must have no
/// behavior (no functions).
///
/// # Example
///
/// ```no_run
/// implement! { for Game =>
///     AcyclicGame,
///     AcyclicallySolvable,
///     TreeSolvable,
///     TierSolvable
/// }
/// ```
///
/// ...which expands to the following:
///
/// ```no_run
/// impl AcyclicallySolvable for Game {}
///
/// impl TreeSolvable for Game {}
///
/// impl TierSolvable for Game {}
/// ```
#[macro_export]
macro_rules! implement {
    (for $b:ty => $($t:ty),+) => {
        $(impl $t for $b { })*
    }
}

/// Syntax sugar. Allows a "literal-like" declaration of collections like
/// `HashSet`s, `HashMap`s, `Vec`s, etc.
///
/// # Example
///
/// ```no_run
/// let s: Vec<_> = collection![1, 2, 3];
/// let s: HashSet<_> = collection! { 1, 2, 3 };
/// let s: HashMap<_, _> = collection! { 1 => 2, 3 => 4 };
/// ```
/// ...which expands to the following:
///
/// ```no_run
/// let s = Vec::from([1, 2, 3]);
/// let s = HashSet::from([1, 2, 3]);
/// let s = HashMap::from([(1, 2), (3, 4)]);
/// ```
#[macro_export]
macro_rules! collection {
    ($($k:expr => $v:expr),* $(,)?) => {{
        core::convert::From::from([$(($k, $v),)*])
    }};
    ($($v:expr),* $(,)?) => {{
        core::convert::From::from([$($v,)*])
    }};
}

/// Syntax sugar. Allows for a declarative way of expressing attribute names,
/// data types, and bit sizes for constructing database schemas.
///
/// # Example
///
/// ```no_run
/// let s1 = schema!("attribute1"; Datatype::CSTR; 16);
///
/// let s2 = schema! {
///     "attribute3"; Datatype::UINT; 20,
///     "attribute4"; Datatype::SINT; 60
/// };
/// ```
///
/// ...which expands to the following:
///
/// ```no_run
/// let s1 = SchemaBuilder::new()
///     .add(Attribute::new("attribute1", Datatype::CSTR, 10))?
///     .build();
///
/// let s2 = SchemaBuilder::new()
///     .add(Attribute::new("attribute3", Datatype::UINT, 20))?
///     .add(Attribute::new("attribute4", Datatype::SINT, 60))?
///     .build();
/// ```
#[macro_export]
macro_rules! schema {
    {$($key:literal; $dt:expr; $value:expr),*} => {
        SchemaBuilder::new()
            $(
                .add(Attribute::new($key, $dt, $value))?
            )*
            .build()
    };
}

/// TODO
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
