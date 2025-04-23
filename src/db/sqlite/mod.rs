//! # SQLite Database Module
//!
//! TODO

use anyhow::Result;
use rusqlite::Statement;
use rusqlite::Transaction;

use crate::game::DEFAULT_STATE_BYTES as DBYTES;
use crate::game::PlayerCount;
use crate::game::State;
use crate::interface::IOMode;
use crate::solver::Solution;

/* MODULES */

#[cfg(test)]
mod test;
mod schema;
mod util;

/* RE-EXPORTS */

pub use schema::Schema;
pub use schema::SchemaBuilder;
pub use util::database;

/* DEFINITIONS */

/// SQL query strings to be prepared into pre-compiled statements.
pub struct Queries {
    pub insert: String,
    pub select: String,
}

/* TRAITS */

pub trait Persistent<const N: PlayerCount, const B: usize = DBYTES> {
    type Queries;

    /// Stores `info` under the key `state`, replacing an existing entry.
    ///
    /// This is used for persistence purposes. More information than `info` may
    /// be stored alongside `info` as a side effect. The effects of this may not
    /// persist unless `commit` is called afterwards.
    ///
    /// # Errors
    ///
    /// When `prepare` is not called before `insert`.
    fn insert(
        &mut self,
        stmt: &mut Statement,
        state: &State<B>,
        info: &Solution<N>,
    ) -> Result<()>;

    /// Retrieves the entry associated with `state`, or `None`.
    ///
    /// Entries are inserted through `insert`. The effects of this may not be
    /// persistent unless `commit` is called afterwards.
    ///
    /// # Errors
    ///
    /// When `prepare` is not called before `select`.
    fn select(
        &mut self,
        stmt: &mut Statement,
        state: &State<B>,
    ) -> Result<Option<Solution<N>>>;

    /// Prepares the underlying store for a series of calls to `insert` and
    /// `select`, according to `mode`.
    ///
    /// # Errors
    ///
    /// On a variety of conditions which depend on the underlying store.
    fn prepare(
        &mut self,
        tx: &mut Transaction,
        mode: IOMode,
    ) -> Result<Self::Queries>;
}
