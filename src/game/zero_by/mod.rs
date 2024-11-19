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

use anyhow::bail;
use anyhow::{Context, Result};
use bitvec::array::BitArray;
use bitvec::field::BitField;
use bitvec::order::Msb0;

use crate::database::{engine, Persistent, ProtoRelational};
use crate::game::error::GameError;
use crate::game::model::Variant;
use crate::game::model::{Player, PlayerCount, State};
use crate::game::zero_by::states::*;
use crate::game::zero_by::variants::*;
use crate::game::Information;
use crate::game::Variable;
use crate::game::{Bounded, Codec, Forward};
use crate::game::{GameData, Transition};
use crate::interface::{IOMode, Solution};
use crate::solver::algorithm::strong;
use crate::solver::model::SUtility;
use crate::solver::{Sequential, SimpleUtility};

/* SUBMODULES */

mod states;
mod variants;

/* DEFINITIONS */

/// The number of elements in the pile (see the game rules).
type Elements = u64;

/* GAME DATA */

const NAME: &str = "zero-by";
const AUTHORS: &str = "Max Fierro <maxfierro@berkeley.edu>";
const ABOUT: &str =
"Many players take turns removing a number of elements from a set of arbitrary \
size. The game variant determines how many players are in the game, how many \
elements are in the set to begin with, and the options players have in the \
amount of elements to remove during their turn. The player who is left with 0 \
elements in their turn loses. A player cannot remove more elements than \
currently available in the set.";

/* GAME IMPLEMENTATION */

pub struct Session {
    start_elems: Elements,
    start_state: State,
    player_bits: usize,
    players: PlayerCount,
    variant: Variant,
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

    pub fn solve<D>(
        &self,
        mode: IOMode,
        method: Solution,
        db: &mut D,
    ) -> Result<()>
    where
        D: ProtoRelational + Persistent,
    {
        match (self.players, method) {
            (2, Solution::Strong) => {
                strong::acyclic::solver::<2, 8, Self, D>(self, mode, db)
                    .context("Failed solver run.")?
            },
            (10, Solution::Strong) => {
                strong::acyclic::solver::<10, 8, Self, D>(self, mode, db)
                    .context("Failed solver run.")?
            },
            _ => bail!(GameError::SolverNotFound {
                input_game_name: NAME,
            }),
        }
        Ok(())
    }

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

/* INFORMATION IMPLEMENTATIONS */

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

/* VARIANCE IMPLEMENTATION */

impl Default for Session {
    fn default() -> Self {
        parse_variant(VARIANT_DEFAULT.to_owned())
            .expect("Failed to parse default state.")
    }
}

impl Variable for Session {
    fn variant(variant: Variant) -> Result<Self> {
        parse_variant(variant).context("Malformed game variant.")
    }

    fn variant_string(&self) -> Variant {
        self.variant.clone()
    }
}

/* TRAVERSAL IMPLEMENTATIONS */

impl Transition for Session {
    fn prograde(&self, state: State) -> Vec<State> {
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

    fn retrograde(&self, state: State) -> Vec<State> {
        let (turn, elements) = self.decode_state(state);
        let mut next = self
            .by
            .iter()
            .map(|&choice| {
                if elements + choice <= self.start_elems {
                    choice
                } else {
                    self.start_elems
                }
            })
            .map(|choice| {
                self.encode_state((turn - 1) % self.players, elements + choice)
            })
            .collect::<Vec<State>>();
        next.sort();
        next.dedup();
        next
    }
}

/* STATE RESOLUTION IMPLEMENTATIONS */

impl Bounded for Session {
    fn start(&self) -> State {
        self.start_state
    }

    fn end(&self, state: State) -> bool {
        let (_, elements) = self.decode_state(state);
        elements == 0
    }
}

impl Codec for Session {
    fn decode(&self, string: String) -> Result<State> {
        Ok(parse_state(self, string)?)
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

/* SOLVING IMPLEMENTATIONS */

impl<const N: PlayerCount> Sequential<N> for Session {
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
