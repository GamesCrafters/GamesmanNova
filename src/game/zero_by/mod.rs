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
use rusqlite::params_from_iter;

use crate::database::Schema;
use crate::database::sled::init_sled;
use crate::database::traits::DrawRecord;
use crate::database::traits::PlayerRecord;
use crate::database::traits::RemotenessRecord;
use crate::database::traits::SQLiteManager;
use crate::database::traits::SimpleUtilityRecord;
use crate::database::traits::SledManager;
use crate::frontend::IOMode;
use crate::game::Component;
use crate::game::GameData;
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::Remoteness;
use crate::game::SUtility;
use crate::game::State;
use crate::game::Variant;
use crate::game::traits::Advance;
use crate::game::traits::Codec;
use crate::game::traits::Implicit;
use crate::game::traits::Information;
use crate::game::traits::Partition;
use crate::game::traits::Sequential;
use crate::game::traits::SimpleUtility;
use crate::game::traits::Transpose;
use crate::game::traits::Variable;
use crate::game::util::min_ubits;
use crate::scheduler::CriticalPathPolicyBuilder;
use crate::scheduler::DashboardLoggerBuilder;
use crate::scheduler::ForwardTaskBuilder;
use crate::scheduler::SchedulerBuilder;
use crate::scheduler::SchedulerContextBuilder;
use crate::scheduler::SchedulerStateBuilder;
use crate::scheduler::TaskBuilder;
use crate::scheduler::ThreadPoolRunnerBuilder;

/* SUBMODULES */

mod states;
mod variants;

/* TYPE ALIASES */

type Elements = u64;
type RemotenessStorage = B32;
type PlayerStorage = B8;

/* CONSTANTS */

// Task hyperparameter -- this is assuming all states below N are reachable. If
// there are N players in the game, states per component will be ~(N * this).
const APROXIMATE_COMPONENT_SIZE: u64 = 10000000;

const NAME: &str = "zero-by";
const AUTHORS: &str = "Max Fierro <maxfierro@berkeley.edu>";
const ABOUT: &str = "Many players take turns removing a number of elements \
from a set of arbitrary size. The game variant determines how many players are \
in the game, how many elements are in the set to begin with, and the options \
players have in the amount of elements to remove during their turn. The player \
who is left with 0 elements in their turn loses. A player cannot remove more \
elements than currently available in the set.";

const VARIANT_DEFAULT: &str = "2-10-1-2";
const VARIANT_PATTERN: &str = r"^[1-9]\d*(?:-[1-9]\d*)+$";
const VARIANT_PROTOCOL: &str = "The variant should be a dash-separated \
group of three or more positive integers. For example, '4-232-23-6-3-6' is \
valid but '598', '-23-1-5', and 'fifteen-2-5' are not. The first integer \
represents the number of players in the game. The second integer represents \
the number of elements in the set. The rest are choices that the players have \
when they need to remove a number of pieces on their turn. Note that the \
numbers can be repeated, but if you repeat the first number it will be a win \
for the player with the first turn in 1 move. If you repeat any of the rest \
of the numbers, the only consequence will be a slight decrease in performance.";

const STATE_DEFAULT: &str = "10-0";
const STATE_PATTERN: &str = r"^\d+-\d+$";
const STATE_PROTOCOL: &str = "Two dash-separated positive integers. The \
first integer indicates the amount of elements left to remove from the set, \
and the second indicates whose turn it is to remove an element. The first \
integer must be less than or equal to the number of initial elements specified \
by the game variant. Likewise, the second integer must be strictly less than \
the number of players in the game.";

/* API STRUCTURES */

#[derive(Clone)]
pub struct Session {
    start_elems: Elements,
    start_state: State,
    player_bits: usize,
    players: PlayerCount,
    sled_db: sled::Db,
    schema: Schema,
    name: String,
    by: Vec<Elements>,
}

pub struct Record<const N: PlayerCount> {
    features: RecordFeatures,
    utility: [SUtility; N],
}

/* PRIVATE STRUCTURES */

#[bitfield]
#[derive(Default)]
struct RecordFeatures {
    remoteness: RemotenessStorage,
    player: PlayerStorage,
}

/* IMPLEMENTATIONS */

impl Session {
    pub fn build(&mut self, mode: IOMode) -> Result<()> {
        self.sled_db = init_sled(mode, self.name())?;
        let executable = match self.players {
            2 => ForwardTaskBuilder::<Self, 2>::default()
                .source(self.source())
                .game(self.clone())
                .threshold(100)
                .build()?,
            _ => bail!("Player count not supported for Zero-By"),
        };

        let about = format!("Forward pass of variant {}", self.name());
        let task = TaskBuilder::default()
            .executable(executable)
            .retriable(true)
            .about(about)
            .build()?;

        let mut scheduler = {
            let policy = CriticalPathPolicyBuilder::default().build()?;
            let runner = ThreadPoolRunnerBuilder::default().build()?;
            let logger = DashboardLoggerBuilder::default().build()?;

            let context = SchedulerContextBuilder::default()
                .policy(policy)
                .logger(logger)
                .runner(runner)
                .build()?;

            let state = SchedulerStateBuilder::default()
                .task(task)
                .build()?;

            SchedulerBuilder::default()
                .context(context)
                .state(state)
                .build()?
        };

        scheduler.run()?;
        Ok(())
    }

    fn encode_state(&self, turn: Player, elements: Elements) -> State {
        let mut state: BitArray<_, Msb0> = BitArray::ZERO;
        state[self.player_bits..].store_be(elements);
        state[..self.player_bits].store_be(turn);
        state.data
    }

    fn decode_state(&self, state: State) -> (Player, Elements) {
        let state: BitArray<_, Msb0> = BitArray::from(state);
        let elements = state[self.player_bits..].load_be::<Elements>();
        let player = state[..self.player_bits].load_be::<Player>();
        (player, elements)
    }
}

/* TRAIT IMPLEMENTATIONS */

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
    fn variant(variant: Option<Variant>) -> Result<Self> {
        variants::parse_variant(variant.unwrap_or(VARIANT_DEFAULT.to_owned()))
            .context("Malformed game variant.")
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Implicit for Session {
    fn outgoing(&self, state: &State) -> Vec<State> {
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

impl Transpose for Session {
    fn incoming(&self, state: &State) -> Vec<State> {
        let (_, start) = self.decode_state(self.start_state);
        let (turn, elements) = self.decode_state(*state);
        let mut prev = self
            .by
            .iter()
            .map(|&choice| {
                if start > elements + choice {
                    elements + choice
                } else {
                    start
                }
            })
            .map(|elements| {
                let turn = (turn - 1) % self.players;
                self.encode_state(turn, elements)
            })
            .collect::<Vec<State>>();

        prev.sort();
        prev.dedup();
        prev
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

impl Advance for Session {
    fn set_verified_start(&mut self, state: &State) {
        self.start_state = *state;
    }
}

impl Partition for Session {
    fn component(&self, state: &State) -> Component {
        let (_turn, elements) = self.decode_state(*state);
        elements / APROXIMATE_COMPONENT_SIZE
    }
}

impl<const N: PlayerCount> Sequential<N> for Session {
    fn turn(&self, state: &State) -> Player {
        let (turn, _elements) = self.decode_state(*state);
        turn
    }
}

impl<const N: PlayerCount> SimpleUtility<N> for Session {
    fn utility(&self, state: &State) -> [SUtility; N] {
        let (turn, _elements) = self.decode_state(*state);
        let mut payoffs = [SUtility::Lose; N];
        payoffs[turn] = SUtility::Win;
        payoffs
    }
}

/* STORAGE IMPLEMENTATIONS */

impl<const N: PlayerCount> SledManager<N> for Session {
    type Record = self::Record<N>;

    fn sled_transaction(&self) -> Result<sled::Tree> {
        self.sled_db
            .open_tree(self.name())
            .context("Failed to open Sled tree for transaction")
    }
}

impl<const N: PlayerCount> SQLiteManager<N> for Session {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn store_lift(
        &mut self,
        state: &State,
        solution: &Record<N>,
        statement: &mut Statement,
    ) -> Result<()> {
        let values = [
            i64::from_be_bytes(*state),
            solution.get_remoteness() as i64,
            solution.get_player() as i64,
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

/* RECORD IMPLEMENTATIONS */

impl<const N: PlayerCount> From<Record<N>> for sled::IVec {
    fn from(val: Record<N>) -> Self {
        let mut bytes = val.features.into_bytes().to_vec();
        let ubits = N * 2;
        let ubytes = ubits.div_ceil(8);

        let mut udata: BitArray<[u8; 8], Msb0> = BitArray::ZERO;
        val.utility
            .iter()
            .enumerate()
            .for_each(|(i, &util)| {
                let start = i * 2;
                udata[start..start + 2].store_be(util as u8);
            });

        bytes.extend_from_slice(&udata.data[..ubytes]);
        bytes.into()
    }
}

impl<const N: PlayerCount> Default for Record<N> {
    fn default() -> Self {
        Self {
            features: Default::default(),
            utility: [Default::default(); N],
        }
    }
}

impl<const N: PlayerCount> RemotenessRecord for Record<N> {
    fn set_remoteness(&mut self, value: Remoteness) -> Result<&mut Self> {
        if min_ubits(value as u128) > RemotenessStorage::BITS {
            bail!("Remoteness {value} would not fit in Sled DB record.")
        }

        self.features
            .set_remoteness(value as u32);

        Ok(self)
    }

    fn get_remoteness(&self) -> Remoteness {
        self.features.remoteness() as u64
    }
}

impl<const N: PlayerCount> SimpleUtilityRecord<N> for Record<N> {
    fn set_utility(&mut self, value: [SUtility; N]) -> Result<&mut Self> {
        self.utility = value;
        Ok(self)
    }

    fn get_utility(&self) -> [SUtility; N] {
        self.utility
    }
}

impl<const N: PlayerCount> PlayerRecord for Record<N> {
    fn set_player(&mut self, value: Player) -> Result<&mut Self> {
        if min_ubits(value as u128) > PlayerStorage::BITS {
            bail!("Remoteness {value} would not fit in Sled DB record.")
        }

        self.features
            .set_player(value as u8);

        Ok(self)
    }

    fn get_player(&self) -> Player {
        self.features.player() as usize
    }
}

impl<const N: PlayerCount> DrawRecord for Record<N> {
    fn set_draw(&mut self, _value: bool) -> Result<&mut Self> {
        Ok(self)
    }

    fn get_draw(&self) -> bool {
        false
    }
}
