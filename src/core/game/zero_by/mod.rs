//! # Zero-By Game Implementations
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use bitvec::array::BitArray;
use bitvec::field::BitField;
use bitvec::order::Msb0;
use modular_bitfield::Specifier;
use rusqlite::Statement;
use rusqlite::Transaction;
use rusqlite::params_from_iter;

use crate::core::game::util::min_ubits;
use crate::traits::database::DrawRecord;
use crate::traits::database::PlayerRecord;
use crate::traits::database::RemotenessRecord;
use crate::traits::database::SQLiteWriter;
use crate::traits::database::SimpleUtilityRecord;
use crate::traits::game::Codec;
use crate::traits::game::Forward;
use crate::traits::game::Implicit;
use crate::traits::game::Information;
use crate::traits::game::Sequential;
use crate::traits::game::SimpleUtility;
use crate::traits::game::Variable;
use crate::types::database::InsertQuery;
use crate::types::frontend::IOMode;
use crate::types::game::GameData;
use crate::types::game::Player;
use crate::types::game::PlayerCount;
use crate::types::game::Remoteness;
use crate::types::game::SUtility;
use crate::types::game::State;
use crate::types::game::Variant;
use crate::types::game::zero_by;
use crate::types::game::zero_by::Elements;
use crate::types::game::zero_by::PlayerStorage;
use crate::types::game::zero_by::Record;
use crate::types::game::zero_by::RemotenessStorage;
use crate::types::game::zero_by::Session;
use crate::types::game::zero_by::VARIANT_DEFAULT;

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
        todo!()
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

/* UTILITY IMPLEMENTATIONS */

impl<const N: PlayerCount> SimpleUtility<N> for Session {
    fn utility(&self, state: &State) -> [SUtility; N] {
        let (turn, _) = self.decode_state(*state);
        let mut payoffs = [SUtility::Lose; N];
        payoffs[turn] = SUtility::Win;
        payoffs
    }
}

/* SQLITE RECORD IMPLEMENTATIONS */

impl<const N: PlayerCount> SQLiteWriter<N> for Session {
    type Solution = Record<N>;
    fn prepare(
        &mut self,
        tx: &mut Transaction,
        mode: IOMode,
    ) -> Result<InsertQuery> {
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

        Ok(self.schema.insert_query())
    }

    fn insert(
        &mut self,
        state: &State,
        solution: &Self::Solution,
        statement: &mut Statement,
    ) -> Result<()> {
        let values = [
            i64::from_be_bytes(*state),
            solution.remoteness() as i64,
            solution.player() as i64,
        ]
        .into_iter()
        .chain(
            solution
                .utility
                .into_iter()
                .map(From::from),
        );

        let params = params_from_iter(values);
        statement.execute(params)?;
        Ok(())
    }
}

/* SLED RECORD IMPLEMENTATIONS */

impl<const N: usize> RemotenessRecord for Record<N> {
    fn set_remoteness(&mut self, value: Remoteness) -> Result<&mut Self> {
        if min_ubits(value as u128) > RemotenessStorage::BITS {
            bail!("Remoteness {value} would not fit in Sled DB record.")
        }

        self.header
            .set_remoteness(value as u32);

        Ok(self)
    }

    fn remoteness(&self) -> Remoteness {
        self.header.remoteness() as u64
    }
}

impl<const N: usize> SimpleUtilityRecord<N> for Record<N> {
    fn set_utility(&mut self, value: [SUtility; N]) -> Result<&mut Self> {
        self.utility = value;
        Ok(self)
    }

    fn utility(&self) -> [SUtility; N] {
        self.utility
    }
}

impl<const N: usize> PlayerRecord for Record<N> {
    fn set_player(&mut self, value: Player) -> Result<&mut Self> {
        if min_ubits(value as u128) > PlayerStorage::BITS {
            bail!("Remoteness {value} would not fit in Sled DB record.")
        }

        self.header.set_player(value as u8);
        Ok(self)
    }

    fn player(&self) -> Player {
        self.header.player() as usize
    }
}

impl<const N: usize> DrawRecord for Record<N> {
    fn set_draw(&mut self, _value: bool) -> Result<&mut Self> {
        Ok(self)
    }

    fn draw(&self) -> bool {
        false
    }
}
