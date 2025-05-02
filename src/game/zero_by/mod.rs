//! # Zero-By Game Module
//!
//! Zero-By is a simple acyclic game where two players take turns removing one
//! of certain amounts of elements from a set of N elements. For example,
//! players could take turns removing either one or two coins from a stack
//! of ten, which would be an instance of Ten to Zero by One or Two (coins).
//!
//! This module encapsulates the commonalities for all Zero-By games, allowing
//! users to specify which abstract instance of the Zero-By game they wish to
//! emulate.

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use bitvec::array::BitArray;
use bitvec::field::BitField;
use bitvec::order::Msb0;
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
use crate::game::zero_by::states::*;
use crate::game::zero_by::variants::*;
use crate::interface::IOMode;
use crate::solver::Game;
use crate::solver::Persistent;
use crate::solver::Queries;
use crate::solver::SUtility;
use crate::solver::SimpleUtility;
use crate::solver::Solution;
use crate::solver::algorithm::acyclic;
use crate::solver::db::Schema;

/* SUBMODULES */

mod states;
mod variants;

/* DEFINITIONS */

type Elements = u64;

/* GAME DATA */

const NAME: &str = "zero-by";
const AUTHORS: &str = "Max Fierro <maxfierro@berkeley.edu>";
const ABOUT: &str = "Many players take turns removing a number of elements \
from a set of arbitrary size. The game variant determines how many players are \
in the game, how many elements are in the set to begin with, and the options \
players have in the amount of elements to remove during their turn. The player \
who is left with 0 elements in their turn loses. A player cannot remove more \
elements than currently available in the set.";

/* GAME IMPLEMENTATION */

pub struct Session {
    start_elems: Elements,
    start_state: State,
    player_bits: usize,
    players: PlayerCount,
    schema: Schema,
    by: Vec<Elements>,
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
        match self.players {
            1 => acyclic::solve::<1, 8, _>(self, mode),
            2 => acyclic::solve::<2, 8, _>(self, mode),
            3 => acyclic::solve::<3, 8, _>(self, mode),
            4 => acyclic::solve::<4, 8, _>(self, mode),
            5 => acyclic::solve::<5, 8, _>(self, mode),
            6 => acyclic::solve::<6, 8, _>(self, mode),
            7 => acyclic::solve::<7, 8, _>(self, mode),
            8 => acyclic::solve::<8, 8, _>(self, mode),
            9 => acyclic::solve::<9, 8, _>(self, mode),
            _ => bail!("Provided player count is not implemented for zero-by."),
        }
    }

    /* UTILITY */

    fn encode_state(&self, turn: Player, elements: Elements) -> State {
        let mut state: BitArray<_, Msb0> = BitArray::ZERO;
        state[..self.player_bits].store_be(turn);
        state[self.player_bits..].store_be(elements);
        state.data
    }

    fn decode_state(&self, state: State) -> (Player, Elements) {
        let state: BitArray<_, Msb0> = BitArray::from(state);
        let player = state[..self.player_bits].load_be::<Player>();
        let elements = state[self.player_bits..].load_be::<Elements>();
        (player, elements)
    }
}

/* IMPLEMENTATIONS */

impl Default for Session {
    fn default() -> Self {
        parse_variant(VARIANT_DEFAULT.to_owned())
            .expect("Failed to parse default variant.")
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

            state_default: STATE_DEFAULT,
            state_pattern: STATE_PATTERN,
            state_protocol: STATE_PROTOCOL,
        }
    }
}

impl Variable for Session {
    fn variant(variant: Variant) -> Result<Self> {
        parse_variant(variant).context("Malformed game variant.")
    }
}

impl Implicit for Session {
    fn adjacent(&self, state: State) -> Vec<State> {
        let (turn, elements) = self.decode_state(state);
        let mut next = self
            .by
            .iter()
            .map(|&choice| if elements <= choice { elements } else { choice })
            .map(|choice| {
                self.encode_state((turn + 1) % self.players, elements - choice)
            })
            .collect::<Vec<State>>();
        next.sort();
        next.dedup();
        next
    }

    fn source(&self) -> State {
        self.start_state
    }

    fn sink(&self, state: State) -> bool {
        let (_, elements) = self.decode_state(state);
        elements == 0
    }
}

impl Codec for Session {
    fn decode(&self, string: String) -> Result<State> {
        decode_state_string(self, string)
    }

    fn encode(&self, state: State) -> Result<String> {
        let (turn, elements) = self.decode_state(state);
        Ok(format!("{elements}-{turn}"))
    }
}

impl Forward for Session {
    fn set_verified_start(&mut self, state: State) {
        self.start_state = state;
    }
}

impl<const N: PlayerCount> Game<N> for Session {
    fn turn(&self, state: State) -> Player {
        let (turn, _) = self.decode_state(state);
        turn
    }
}

impl<const N: PlayerCount> SimpleUtility<N> for Session {
    fn utility(&self, state: State) -> [SUtility; N] {
        let (turn, _) = self.decode_state(state);
        let mut payoffs = [SUtility::Lose; N];
        payoffs[turn] = SUtility::Win;
        payoffs
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
