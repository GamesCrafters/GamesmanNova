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
//!
//! #### Authorship
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};
use bitvec::field::BitField;
use states::*;

use crate::game::error::GameError;
use crate::game::zero_by::variants::*;
use crate::game::{util, Bounded, Codec, Forward};
use crate::game::{Game, GameData, Transition};
use crate::interface::{IOMode, SolutionMode};
use crate::model::database::Identifier;
use crate::model::game::{Player, PlayerCount, State};
use crate::model::solver::SUtility;
use crate::solver::algorithm::strong;
use crate::solver::{Extensive, SimpleUtility};

/* SUBMODULES */

mod states;
mod variants;

/* DEFINITIONS */

/// The number of elements in the pile (see the game rules).
type Elements = u64;

/* GAME DATA */

const NAME: &'static str = "zero-by";
const AUTHORS: &'static str = "Max Fierro <maxfierro@berkeley.edu>";
const ABOUT: &'static str =
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
    variant: String,
    by: Vec<Elements>,
}

impl Game for Session {
    fn new(variant: Option<String>) -> Result<Self> {
        if let Some(v) = variant {
            parse_variant(v).context("Malformed game variant.")
        } else {
            Ok(parse_variant(VARIANT_DEFAULT.to_owned()).unwrap())
        }
    }

    fn id(&self) -> Identifier {
        todo!()
    }

    fn info(&self) -> GameData {
        GameData {
            variant: self.variant.clone(),

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

    fn solve(&self, mode: IOMode, method: SolutionMode) -> Result<()> {
        match (self.players, method) {
            (2, SolutionMode::Strong) => {
                strong::acyclic::solver::<2, 8, Self>(self, mode)
                    .context("Failed solver run.")?
            },
            (10, SolutionMode::Strong) => {
                strong::acyclic::solver::<10, 8, Self>(self, mode)
                    .context("Failed solver run.")?
            },
            _ => {
                return Err(GameError::SolverNotFound {
                    input_game_name: NAME,
                })
                .context("Solver not found.");
            },
        }
        Ok(())
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
        elements <= 0
    }
}

impl Codec for Session {
    fn decode(&self, string: String) -> Result<State> {
        Ok(parse_state(&self, string)?)
    }

    fn encode(&self, state: State) -> String {
        let (turn, elements) = self.decode_state(state);
        format!("{}-{}", elements, turn)
    }
}

impl Forward for Session {
    fn forward(&mut self, history: Vec<String>) -> Result<()> {
        self.start_state = util::verify_history_dynamic(self, history)
            .context("Malformed game state encoding.")?;
        Ok(())
    }
}

/* SOLVING IMPLEMENTATIONS */

impl<const N: PlayerCount> Extensive<N> for Session {
    fn turn(&self, state: State) -> Player {
        let (turn, _) = self.decode_state(state);
        turn
    }
}

impl<const N: PlayerCount> SimpleUtility<N> for Session {
    fn utility(&self, state: State) -> [SUtility; N] {
        let (turn, _) = self.decode_state(state);
        let mut payoffs = [SUtility::LOSE; N];
        payoffs[turn] = SUtility::WIN;
        payoffs
    }
}

/* UTILITY FUNCTIONS */

impl Session {
    fn encode_state(&self, turn: Player, elements: Elements) -> State {
        let mut state = State::ZERO;
        state[..self.player_bits].store_be(turn);
        state[self.player_bits..].store_be(elements);
        state
    }

    fn decode_state(&self, state: State) -> (Player, Elements) {
        let player = state[..self.player_bits].load_be::<Player>();
        let elements = state[self.player_bits..].load_be::<Elements>();
        (player, elements)
    }
}
