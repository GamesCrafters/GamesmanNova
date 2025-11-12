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
use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;
use rusqlite::Statement;
use rusqlite::Transaction;
use rusqlite::params_from_iter;

use crate::database::InsertQuery;
use crate::database::Schema;
use crate::database::traits::DrawRecord;
use crate::database::traits::PlayerRecord;
use crate::database::traits::RemotenessRecord;
use crate::database::traits::SQLiteWriter;
use crate::database::traits::SimpleUtilityRecord;
use crate::frontend::IOMode;
use crate::game::GameData;
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::Remoteness;
use crate::game::SUtility;
use crate::game::State;
use crate::game::Variant;
use crate::game::traits::Codec;
use crate::game::traits::Forward;
use crate::game::traits::Implicit;
use crate::game::traits::Information;
use crate::game::traits::Sequential;
use crate::game::traits::SimpleUtility;
use crate::game::traits::Variable;
use crate::game::util::min_ubits;

/* SUBMODULES */

mod states;
mod variants;

/* TYPE ALIASES */

pub type Elements = u64;
pub type RemotenessStorage = B32;
pub type PlayerStorage = B8;

/* CONSTANTS */

pub const NAME: &str = "zero-by";
pub const AUTHORS: &str = "Max Fierro <maxfierro@berkeley.edu>";
pub const ABOUT: &str = "Many players take turns removing a number of elements \
from a set of arbitrary size. The game variant determines how many players are \
in the game, how many elements are in the set to begin with, and the options \
players have in the amount of elements to remove during their turn. The player \
who is left with 0 elements in their turn loses. A player cannot remove more \
elements than currently available in the set.";

pub const VARIANT_DEFAULT: &str = "2-10-1-2";
pub const VARIANT_PATTERN: &str = r"^[1-9]\d*(?:-[1-9]\d*)+$";
pub const VARIANT_PROTOCOL: &str = "The variant should be a dash-separated \
group of three or more positive integers. For example, '4-232-23-6-3-6' is \
valid but '598', '-23-1-5', and 'fifteen-2-5' are not. The first integer \
represents the number of players in the game. The second integer represents \
the number of elements in the set. The rest are choices that the players have \
when they need to remove a number of pieces on their turn. Note that the \
numbers can be repeated, but if you repeat the first number it will be a win \
for the player with the first turn in 1 move. If you repeat any of the rest \
of the numbers, the only consequence will be a slight decrease in performance.";

pub const STATE_DEFAULT: &str = "10-0";
pub const STATE_PATTERN: &str = r"^\d+-\d+$";
pub const STATE_PROTOCOL: &str = "Two dash-separated positive integers. The \
first integer indicates the amount of elements left to remove from the set, \
and the second indicates whose turn it is to remove an element. The first \
integer must be less than or equal to the number of initial elements specified \
by the game variant. Likewise, the second integer must be strictly less than \
the number of players in the game.";

/* STRUCTURES */

pub struct Session {
    pub start_elems: Elements,
    pub start_state: State,
    pub player_bits: usize,
    pub players: PlayerCount,
    pub schema: Schema,
    pub name: String,
    pub by: Vec<Elements>,
}

#[bitfield]
pub struct RecordHeader {
    pub remoteness: RemotenessStorage,
    pub player: PlayerStorage,
}

pub struct Record<const N: PlayerCount> {
    pub header: RecordHeader,
    pub utility: [SUtility; N],
}

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
