//! # Database Traits
//!
//! TODO

use anyhow::Result;
use rusqlite::Statement;
use rusqlite::Transaction;

use crate::database::InsertQuery;
use crate::database::SelectQuery;
use crate::frontend::IOMode;
use crate::game::DEFAULT_STATE_BYTES;
use crate::game::IUtility;
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::Remoteness;
use crate::game::SUtility;
use crate::game::State;

/* SQLITE INTERFACES */

pub trait SQLiteWriter<
    const N: PlayerCount,
    const B: usize = DEFAULT_STATE_BYTES,
>
{
    type Solution;

    fn prepare(
        &mut self,
        tx: &mut Transaction,
        mode: IOMode,
    ) -> Result<InsertQuery>;

    fn insert(
        &mut self,
        state: &State<B>,
        solution: &Self::Solution,
        statement: &mut Statement,
    ) -> Result<()>;
}

pub trait SQLiteReader<
    const N: PlayerCount,
    const B: usize = DEFAULT_STATE_BYTES,
>
{
    type Solution;

    fn prepare(&mut self, tx: &mut Transaction) -> Result<SelectQuery>;

    fn insert(
        &mut self,
        state: &State<B>,
        statement: &mut Statement,
    ) -> Result<Self::Solution>;
}

/* SLED INTERFACES */

pub trait RemotenessRecord: Sized {
    fn set_remoteness(&mut self, value: Remoteness) -> Result<&mut Self>;
    fn remoteness(&self) -> Remoteness;
}

pub trait PlayerRecord: Sized {
    fn set_player(&mut self, value: Player) -> Result<&mut Self>;
    fn player(&self) -> Player;
}

pub trait DrawRecord: Sized {
    fn set_draw(&mut self, value: bool) -> Result<&mut Self>;
    fn draw(&self) -> bool;
}

/* SLED UTILITY INTERFACES */

pub trait IntegerUtilityRecord<const N: PlayerCount>: Sized {
    fn set_utility(&mut self, value: [IUtility; N]) -> Result<&mut Self>;
    fn utility(&self) -> [IUtility; N];
}

pub trait SimpleUtilityRecord<const N: PlayerCount>: Sized {
    fn set_utility(&mut self, value: [SUtility; N]) -> Result<&mut Self>;
    fn utility(&self) -> [SUtility; N];
}

pub trait ClassicUtilityRecord: Sized {
    fn set_utility(&mut self, value: SUtility) -> Result<&mut Self>;
    fn utility(&self) -> SUtility;
}

pub trait PuzzleUtilityRecord: Sized {
    fn set_utility(&mut self, value: SUtility) -> Result<&mut Self>;
    fn utility(&self) -> SUtility;
}
