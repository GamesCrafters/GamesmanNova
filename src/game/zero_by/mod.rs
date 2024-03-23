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
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};
use states::*;

use crate::game::error::GameError;
use crate::game::zero_by::variants::*;
use crate::game::{util, Acyclic, Bounded, Legible};
use crate::game::{DTransition, Game, GameData, Solvable};
use crate::implement;
use crate::interface::{IOMode, SolutionMode};
use crate::model::PlayerCount;
use crate::model::Utility;
use crate::model::{State, Turn};
use crate::solver::strong;

use super::util::unpack_turn;

/* SUBMODULES */

mod states;
mod variants;

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
    variant: String,
    players: PlayerCount,
    start: State,
    by: Vec<u64>,
}

impl Game for Session {
    fn initialize(variant: Option<String>) -> Result<Self> {
        if let Some(v) = variant {
            parse_variant(v).context("Malformed game variant.")
        } else {
            Ok(parse_variant(VARIANT_DEFAULT.to_owned()).unwrap())
        }
    }

    fn id(&self) -> String {
        format!("{}.{}", NAME, self.variant)
    }

    fn forward(&mut self, history: Vec<String>) -> Result<()> {
        self.start = util::verify_history_dynamic(self, history)
            .context("Malformed game state encoding.")?;
        Ok(())
    }

    fn info(&self) -> GameData {
        GameData {
            variant: &self.variant,

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
                strong::acyclic::dynamic_solver::<2, Self>(self, mode)
                    .context("Failed solver run.")?
            },
            (10, SolutionMode::Strong) => {
                strong::acyclic::dynamic_solver::<10, Self>(self, mode)
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

/* TRAVERSAL DECLARATIONS */

impl Bounded<State> for Session {
    fn start(&self) -> State {
        self.start
    }

    fn end(&self, state: State) -> bool {
        state == 0
    }
}

impl DTransition<State> for Session {
    fn prograde(&self, state: State) -> Vec<State> {
        let (state, turn) = util::unpack_turn(state, self.players);
        let mut next = self
            .by
            .iter()
            .map(|&choice| if state <= choice { state } else { choice })
            .map(|choice| {
                util::pack_turn(
                    state - choice,
                    (turn + 1) % self.players,
                    self.players,
                )
            })
            .collect::<Vec<State>>();
        next.sort();
        next.dedup();
        next
    }

    fn retrograde(&self, state: State) -> Vec<State> {
        let (state, turn) = util::unpack_turn(state, self.players);
        let mut next =
            self.by
                .iter()
                .map(|&choice| {
                    if state + choice <= self.start {
                        choice
                    } else {
                        self.start
                    }
                })
                .map(|choice| {
                    util::pack_turn(
                        state + choice,
                        (turn - 1) % self.players,
                        self.players,
                    )
                })
                .collect::<Vec<State>>();
        next.sort();
        next.dedup();
        next
    }
}

/* SUPPLEMENTAL DECLARATIONS */

impl Legible<State> for Session {
    fn decode(&self, string: String) -> Result<State> {
        Ok(parse_state(&self, string)?)
    }

    fn encode(&self, state: State) -> String {
        let (elements, turn) = util::unpack_turn(state, self.players);
        format!("{}-{}", elements, turn)
    }
}

/* SOLVING DECLARATIONS */

implement! { for Session =>
    Acyclic<2>,
    Acyclic<10>
}

impl Solvable<2> for Session {
    fn utility(&self, state: State) -> [Utility; 2] {
        let (_, turn) = unpack_turn(state, 2);
        let mut payoffs = [-1; 2];
        payoffs[turn] = 1;
        payoffs
    }

    fn turn(&self, state: State) -> Turn {
        util::unpack_turn(state, 2).1
    }
}

impl Solvable<10> for Session {
    fn utility(&self, state: State) -> [Utility; 10] {
        let (_, turn) = unpack_turn(state, 10);
        let mut payoffs = [-1; 10];
        payoffs[turn] = 9;
        payoffs
    }

    fn turn(&self, state: State) -> Turn {
        util::unpack_turn(state, 10).1
    }
}
