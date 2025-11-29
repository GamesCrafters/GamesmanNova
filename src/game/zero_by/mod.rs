//! # Zero-By Game Implementations
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::vec::BitVec;
use modular_bitfield::Specifier;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;

use crate::database::traits::DrawRecord;
use crate::database::traits::PlayerRecord;
use crate::database::traits::RemotenessRecord;
use crate::database::traits::SimpleUtilityRecord;
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

/* SUBMODULES */

mod states;
mod variants;

/* TYPE ALIASES */

type Elements = u64;
type RemotenessStorage = B32;
type PlayerStorage = B8;

/* CONSTANTS */

const APROXIMATE_COMPONENT_SIZE: u64 = 50000;
const FEATURES_BYTES: usize =
    (RemotenessStorage::BITS + PlayerStorage::BITS).div_ceil(8);

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
pub struct Ruleset {
    start_elems: Elements,
    start_state: State,
    player_bits: usize,
    players: PlayerCount,
    name: String,
    by: Vec<Elements>,
}

#[derive(Clone, Default)]
pub struct Record {
    features: RecordFeatures,
    utility: Vec<SUtility>,
}

/* PRIVATE STRUCTURES */

#[bitfield]
#[derive(Clone, Default)]
struct RecordFeatures {
    remoteness: RemotenessStorage,
    player: PlayerStorage,
}

/* IMPLEMENTATIONS */

impl Ruleset {
    /* Helper methods may be added here in the future */
}

impl Ruleset {
    fn encode_state(&self, turn: Player, elements: Elements) -> State {
        let mut state: BitVec<u8, Msb0> = BitVec::repeat(false, 64);
        state[self.player_bits..].store_be(elements);
        state[..self.player_bits].store_be(turn);
        state
    }

    fn decode_state(&self, state: &State) -> (Player, Elements) {
        let elements = state[self.player_bits..].load_be::<Elements>();
        let player = state[..self.player_bits].load_be::<Player>();
        (player, elements)
    }
}

/* TRAIT IMPLEMENTATIONS */

impl Information for Ruleset {
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

impl Variable for Ruleset {
    fn variant(variant: Option<Variant>) -> Result<Self> {
        variants::parse_variant(variant.unwrap_or(VARIANT_DEFAULT.to_owned()))
            .context("Malformed game variant.")
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Implicit for Ruleset {
    fn outgoing(&self, state: &State) -> Vec<State> {
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
        self.start_state.clone()
    }

    fn sink(&self, state: &State) -> bool {
        let (_, elements) = self.decode_state(state);
        elements == 0
    }
}

impl Transpose for Ruleset {
    fn incoming(&self, state: &State) -> Vec<State> {
        let (_, start) = self.decode_state(&self.start_state);
        let (turn, elements) = self.decode_state(state);
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

impl Codec for Ruleset {
    fn decode(&self, string: String) -> Result<State> {
        Ok(states::parse_state(self, string)?)
    }

    fn encode(&self, state: &State) -> Result<String> {
        let (turn, elements) = self.decode_state(state);
        Ok(format!("{elements}-{turn}"))
    }
}

impl Advance for Ruleset {
    fn set_verified_start(&mut self, state: &State) {
        self.start_state = state.clone();
    }
}

impl Partition for Ruleset {
    fn component(&self, state: &State) -> Component {
        let (_turn, elements) = self.decode_state(state);
        elements / APROXIMATE_COMPONENT_SIZE
    }
}

impl<const N: PlayerCount> Sequential<N> for Ruleset {
    fn turn(&self, state: &State) -> Player {
        let (turn, _elements) = self.decode_state(state);
        turn
    }
}

impl<const N: PlayerCount> SimpleUtility<N> for Ruleset {
    fn utility(&self, state: &State) -> [SUtility; N] {
        let (turn, _elements) = self.decode_state(state);
        let mut payoffs = [SUtility::Lose; N];
        payoffs[turn] = SUtility::Win;
        payoffs
    }
}

/* RECORD IMPLEMENTATIONS */

impl From<Record> for Vec<u8> {
    fn from(val: Record) -> Self {
        let mut bytes = val.features.into_bytes().to_vec();
        let n = val.utility.len();
        let ubits = n * 2;
        let ubytes = ubits.div_ceil(8);

        let mut udata: BitVec<u8, Msb0> = BitVec::repeat(false, 64);
        val.utility
            .iter()
            .enumerate()
            .for_each(|(i, &util)| {
                let start = i * 2;
                udata[start..start + 2].store_be(util as u8);
            });

        bytes.extend_from_slice(&udata.as_raw_slice()[..ubytes]);
        bytes
    }
}

impl TryFrom<Vec<u8>> for Record {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self> {
        if bytes.len() < FEATURES_BYTES {
            bail!("Insufficient bytes for Record features");
        }

        let farray: [u8; FEATURES_BYTES] =
            bytes[..FEATURES_BYTES].try_into()?;
        let features = RecordFeatures::from_bytes(farray);

        let ubytes = &bytes[FEATURES_BYTES..];
        let udata = BitVec::<u8, Msb0>::from_slice(ubytes);
        let n = ubytes.len() * 8 / 2;

        let utility = (0..n)
            .map(|i| {
                let start = i * 2;
                let value: u8 = udata[start..start + 2].load_be();
                SUtility::try_from(value)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { features, utility })
    }
}

impl RemotenessRecord for Record {
    fn set_remoteness(&mut self, value: Remoteness) -> Result<&mut Self> {
        if min_ubits(value as u128) > RemotenessStorage::BITS {
            bail!("Remoteness {value} would not fit in RocksDB record.")
        }

        self.features
            .set_remoteness(value as u32);

        Ok(self)
    }

    fn get_remoteness(&self) -> Remoteness {
        self.features.remoteness() as u64
    }
}

impl SimpleUtilityRecord for Record {
    fn set_utility(&mut self, value: Vec<SUtility>) -> Result<&mut Self> {
        self.utility = value;
        Ok(self)
    }

    fn get_utility(&self) -> Vec<SUtility> {
        self.utility.clone()
    }
}

impl PlayerRecord for Record {
    fn set_player(&mut self, value: Player) -> Result<&mut Self> {
        if min_ubits(value as u128) > PlayerStorage::BITS {
            bail!("Player {value} would not fit in RocksDB record.")
        }

        self.features
            .set_player(value as u8);

        Ok(self)
    }

    fn get_player(&self) -> Player {
        self.features.player() as usize
    }
}

impl DrawRecord for Record {
    fn set_draw(&mut self, _value: bool) -> Result<&mut Self> {
        Ok(self)
    }

    fn get_draw(&self) -> bool {
        false
    }
}
