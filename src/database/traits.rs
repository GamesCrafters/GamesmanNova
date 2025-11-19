//! # Database Traits
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use rusqlite::Statement;
use rusqlite::Transaction;
use sled::IVec;

use crate::database::Schema;
use crate::game::DEFAULT_STATE_BYTES;
use crate::game::IUtility;
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::Remoteness;
use crate::game::SUtility;
use crate::game::State;

/* SOLVER STORAGE INTERFACES */

pub trait SledManager<
    const N: PlayerCount,
    const B: usize = DEFAULT_STATE_BYTES,
>
{
    type Record: Into<IVec>
        + IntegerUtilityRecord<N>
        + RemotenessRecord
        + PlayerRecord
        + DrawRecord
        + Default;

    fn sled_transaction(&self) -> Result<sled::Tree>;
}

pub trait RemotenessRecord: Sized {
    fn set_remoteness(&mut self, value: Remoteness) -> Result<&mut Self>;
    fn get_remoteness(&self) -> Remoteness;
}

pub trait PlayerRecord: Sized {
    fn set_player(&mut self, value: Player) -> Result<&mut Self>;
    fn get_player(&self) -> Player;
}

pub trait DrawRecord: Sized {
    fn set_draw(&mut self, value: bool) -> Result<&mut Self>;
    fn get_draw(&self) -> bool;
}

pub trait IntegerUtilityRecord<const N: PlayerCount>: Sized {
    fn set_utility(&mut self, value: [IUtility; N]) -> Result<&mut Self>;
    fn get_utility(&self) -> [IUtility; N];
}

pub trait SimpleUtilityRecord<const N: PlayerCount>: Sized {
    fn set_utility(&mut self, value: [SUtility; N]) -> Result<&mut Self>;
    fn get_utility(&self) -> [SUtility; N];
}

pub trait ClassicUtilityRecord: Sized {
    fn set_utility(&mut self, value: SUtility) -> Result<&mut Self>;
    fn get_utility(&self) -> SUtility;
}

pub trait PuzzleUtilityRecord: Sized {
    fn set_utility(&mut self, value: SUtility) -> Result<&mut Self>;
    fn get_utility(&self) -> SUtility;
}

/* DATASET GENERATION INTERFACES */

pub trait SQLiteManager<
    const N: PlayerCount,
    const B: usize = DEFAULT_STATE_BYTES,
> where
    Self: SledManager<N, B>,
{
    fn schema(&self) -> &Schema;

    fn store_lift(
        &mut self,
        state: &State<B>,
        solution: &Self::Record,
        statement: &mut Statement,
    ) -> Result<()>;

    fn sqlite_transaction<'a>(
        &self,
        conn: &'a mut rusqlite::Connection,
    ) -> Result<Transaction<'a>> {
        let tx = conn
            .transaction()
            .context("Failed to start SQLite transaction")?;

        tx.execute(&self.schema().create_table_query(), [])
            .context("Failed to create SQLite table")?;

        Ok(tx)
    }
}
