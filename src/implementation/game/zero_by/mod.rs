//! # Zero-By Game Implementations
//!
//! TODO

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

use crate::interface::database::Persistent;
use crate::interface::game::Codec;
use crate::interface::game::Forward;
use crate::interface::game::Implicit;
use crate::interface::game::Information;
use crate::interface::game::Sequential;
use crate::interface::game::SimpleUtility;
use crate::interface::game::Variable;
use crate::model::database::Queries;
use crate::model::frontend::IOMode;
use crate::model::game::GameData;
use crate::model::game::Player;
use crate::model::game::PlayerCount;
use crate::model::game::SUtility;
use crate::model::game::State;
use crate::model::game::Variant;
use crate::model::game::zero_by;
use crate::model::game::zero_by::Elements;
use crate::model::game::zero_by::Session;
use crate::model::game::zero_by::VARIANT_DEFAULT;
use crate::model::solver::Solution;

/* UTILITY SUBMODULES */

mod states;
mod variants;

/* IMPLEMENTATIONS */

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

impl Default for Session {
    fn default() -> Self {
        variants::parse_variant(VARIANT_DEFAULT.to_owned())
            .expect("Failed to parse default state.")
    }
}

impl Information for Session {
    fn info() -> GameData {
        GameData {
            name: zero_by::NAME,
            authors: zero_by::AUTHORS,
            about: zero_by::ABOUT,

            variant_protocol: zero_by::VARIANT_PROTOCOL,
            variant_pattern: zero_by::VARIANT_PATTERN,
            variant_default: zero_by::VARIANT_DEFAULT,

            state_default: zero_by::STATE_DEFAULT,
            state_pattern: zero_by::STATE_PATTERN,
            state_protocol: zero_by::STATE_PROTOCOL,
        }
    }
}

impl Variable for Session {
    fn variant(variant: Variant) -> Result<Self> {
        variants::parse_variant(variant).context("Malformed game variant.")
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Implicit for Session {
    fn adjacent(&self, state: &State) -> Vec<State> {
        let (turn, elements) = self.decode_state(*state);
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

    fn sink(&self, state: &State) -> bool {
        let (_, elements) = self.decode_state(*state);
        elements == 0
    }
}

impl Codec for Session {
    fn decode(&self, string: String) -> Result<State> {
        Ok(states::parse_state(self, string)?)
    }

    fn encode(&self, state: &State) -> Result<String> {
        let (turn, elements) = self.decode_state(*state);
        Ok(format!("{elements}-{turn}"))
    }
}

impl Forward for Session {
    fn set_verified_start(&mut self, state: &State) {
        self.start_state = *state;
    }
}

impl<const N: PlayerCount> Sequential<N> for Session {
    fn turn(&self, state: &State) -> Player {
        let (turn, _) = self.decode_state(*state);
        turn
    }
}

impl<const N: PlayerCount> SimpleUtility<N> for Session {
    fn utility(&self, state: &State) -> [SUtility; N] {
        let (turn, _) = self.decode_state(*state);
        let mut payoffs = [SUtility::Lose; N];
        payoffs[turn] = SUtility::Win;
        payoffs
    }
}

impl<const N: PlayerCount> Persistent<N> for Session {
    type QueryOptions = Queries;

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
