//! # MNK Game Module
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use rusqlite::Error::QueryReturnedNoRows;
use rusqlite::Statement;
use rusqlite::Transaction;
use rusqlite::params_from_iter;

use crate::game::Codec;
use crate::game::Forward;
use crate::game::GameData;
use crate::game::Implicit;
use crate::game::Information;
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::State;
use crate::game::Variable;
use crate::game::Variant;
use crate::game::mnk::states::*;
use crate::game::mnk::variants::*;
use crate::interface::IOMode;
use crate::solver::Game;
use crate::solver::Persistent;
use crate::solver::Queries;
use crate::solver::SUtility;
use crate::solver::SimpleUtility;
use crate::solver::Solution;
use crate::solver::db::Schema;

/* SUBMODULES */

mod states;
mod variants;

/* GAME DATA */

const NAME: &str = "mnk";
const AUTHORS: &str = "Max Fierro <maxfierro@berkeley.edu>";
const ABOUT: &str = "TODO";

/* GAME IMPLEMENTATION */

pub struct Session {
    schema: Schema,
}

impl Session {
    pub fn new(variant: Option<Variant>) -> Result<Self> {
        if let Some(v) = variant {
            Self::variant(v)
        } else {
            Ok(Self::default())
        }
    }

    pub fn solve(&mut self, mode: IOMode) -> Result<()> {
        todo!()
    }
}

/* IMPLEMENTATIONS */

impl Default for Session {
    fn default() -> Self {
        todo!()
    }
}

impl Information for Session {
    fn info() -> GameData {
        GameData {
            name: NAME,
            authors: AUTHORS,
            about: ABOUT,

            variant_protocol: VARIANT_PROTOCOL,
            variant_pattern: VARIANT_PATTERN,
            variant_default: VARIANT_DEFAULT,

            state_pattern: STATE_PATTERN,
            state_default: STATE_DEFAULT,
            state_protocol: STATE_PROTOCOL,
        }
    }
}

impl Variable for Session {
    fn variant(variant: Variant) -> Result<Self> {
        todo!()
    }
}

impl Implicit for Session {
    fn adjacent(&self, state: State) -> Vec<State> {
        todo!()
    }

    fn source(&self) -> State {
        todo!()
    }

    fn sink(&self, state: State) -> bool {
        todo!()
    }
}

impl Codec for Session {
    fn decode(&self, string: String) -> Result<State> {
        todo!()
    }

    fn encode(&self, state: State) -> Result<String> {
        todo!()
    }
}

impl Forward for Session {
    fn set_verified_start(&mut self, state: State) {
        todo!()
    }
}

impl<const N: PlayerCount> Game<N> for Session {
    fn turn(&self, state: State) -> Player {
        todo!()
    }
}

impl<const N: PlayerCount> SimpleUtility<N> for Session {
    fn utility(&self, state: State) -> [SUtility; N] {
        todo!()
    }
}

impl<const N: PlayerCount> Persistent<N> for Session {
    fn prepare(
        &mut self,
        tx: &mut Transaction,
        mode: IOMode,
    ) -> Result<Queries> {
        let drop_sql = self.schema.drop_table_query();
        let create_sql = self.schema.create_table_query();
        match mode {
            IOMode::Constructive | IOMode::Forgetful => (),
            IOMode::Overwrite => {
                tx.execute(&drop_sql, [])
                    .context("Failed to drop existing table")?;
            },
        }

        tx.execute(&create_sql, [])
            .context("Failed to create table")?;

        let insert = self.schema.insert_query();
        let select = self.schema.select_query();
        let queries = Queries { insert, select };

        Ok(queries)
    }

    fn insert(
        &mut self,
        stmt: &mut Statement,
        state: &State,
        info: &Solution<N>,
    ) -> Result<()> {
        stmt.execute(params_from_iter(
            [
                i64::from_be_bytes(*state),
                info.remoteness as i64,
                info.player as i64,
            ]
            .iter()
            .chain(info.utility.iter()),
        ))?;
        Ok(())
    }

    fn select(
        &mut self,
        stmt: &mut Statement,
        state: &State,
    ) -> Result<Option<Solution<N>>> {
        let start = self.schema.utility_index();
        let row = stmt.query_row([i64::from_be_bytes(*state)], |row| {
            let mut utility: [i64; N] = [0; N];
            for (i, item) in utility.iter_mut().enumerate() {
                *item = row.get(start + i)?;
            }

            Ok(Solution {
                remoteness: row.get(1)?,
                utility,
                player: row.get(2)?,
            })
        });

        match row {
            Err(QueryReturnedNoRows) => Ok(None),
            Ok(data) => Ok(Some(data)),
            Err(e) => Err(anyhow!(e)),
        }
    }
}
