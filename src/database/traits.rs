//! # Database Traits
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use rocksdb::DB;
use rusqlite::Statement;
use rusqlite::Transaction;

use std::sync::Arc;

use crate::database::Schema;
use crate::game::IUtility;
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::Remoteness;
use crate::game::SUtility;
use crate::game::State;

pub trait RocksDBManager<const N: PlayerCount> {
    type Record: Into<Vec<u8>>
        + IntegerUtilityRecord
        + RemotenessRecord
        + PlayerRecord
        + DrawRecord
        + Default;

    fn rocksdb_transaction(&self) -> Result<Arc<DB>>;
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

pub trait IntegerUtilityRecord: Sized {
    fn set_utility(&mut self, value: Vec<IUtility>) -> Result<&mut Self>;
    fn get_utility(&self) -> Vec<IUtility>;
}

pub trait SimpleUtilityRecord: Sized {
    fn set_utility(&mut self, value: Vec<SUtility>) -> Result<&mut Self>;
    fn get_utility(&self) -> Vec<SUtility>;
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

pub trait SQLiteManager<const N: PlayerCount>
where
    Self: RocksDBManager<N>,
{
    fn schema(&self) -> &Schema;

    fn store_lift(
        &mut self,
        state: &State,
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
