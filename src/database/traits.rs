//! # Database Traits
//!
//! TODO

use anyhow::Result;
use rusqlite::Statement;

use crate::database::Schema;
use crate::game::DEFAULT_STATE_BYTES;
use crate::game::IUtility;
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::Remoteness;
use crate::game::SUtility;
use crate::game::State;

/* SLED INTERFACES */

pub trait SledManager<
    const N: PlayerCount,
    const B: usize = DEFAULT_STATE_BYTES,
>
{
    type SolutionRecord: Default;
    fn read(&self, state: &State<B>) -> Result<Self::SolutionRecord>;
    fn write(
        &self,
        state: &State<B>,
        record: &Self::SolutionRecord,
    ) -> Result<()>;
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

/* SQLITE INTERFACES */

pub trait SQLiteManager<const N: PlayerCount> {
    type SolutionRecord;
    fn schema(&self) -> &Schema;
}

pub trait SQLiteWriter<
    SolutionRecord,
    const N: PlayerCount,
    const B: usize = DEFAULT_STATE_BYTES,
> where
    Self: SQLiteManager<N, SolutionRecord = SolutionRecord>,
{
    fn insert(
        &mut self,
        state: &State<B>,
        solution: &SolutionRecord,
        statement: &mut Statement,
    ) -> Result<()>;
}
