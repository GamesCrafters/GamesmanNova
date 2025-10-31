//! # Database Interfaces
//!
//! TODO

use anyhow::Result;
use rusqlite::Statement;
use rusqlite::Transaction;

use crate::model::database::Queries;
use crate::model::frontend::IOMode;
use crate::model::game::DEFAULT_STATE_BYTES;
use crate::model::game::PlayerCount;
use crate::model::game::State;
use crate::model::solver::Solution;

/* SQLITE INTERFACES */

pub trait Persistent<const N: PlayerCount, const B: usize = DEFAULT_STATE_BYTES>
{
    type QueryOptions;

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
    ) -> Result<Queries>;
}
