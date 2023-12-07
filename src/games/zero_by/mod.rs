//! # Zero-By Game Module
//!
//! Zero-By is a small acyclic game, where two players take turns removing
//! one of certain amounts of elements from a set of N elements. For example,
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

use super::utils;
use super::Acyclic;
use super::Legible;
use crate::games::{DynamicAutomaton, Game, GameData, Solvable};
use crate::implement;
use crate::interfaces::terminal::cli::IOMode;
use crate::interfaces::terminal::cli::Solution;
use crate::models::PlayerCount;
use crate::models::Utility;
use crate::solvers::strong::acyclic;
use crate::{
    errors::NovaError,
    models::{State, Turn},
};
use nalgebra::SVector;
use states::*;
use variants::*;

/* SUBMODULES */

mod states;
mod variants;

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
    variant: String,
    players: PlayerCount,
    start: State,
    by: Vec<u64>,
}

impl Game for Session {
    fn initialize(variant: Option<String>) -> Result<Self, NovaError> {
        if let Some(v) = variant {
            parse_variant(v)
        } else {
            parse_variant(VARIANT_DEFAULT.to_owned())
        }
    }

    fn id(&self) -> String {
        format!("{}.{}", NAME, self.variant)
    }

    fn forward(&mut self, history: Vec<String>) -> Result<(), NovaError> {
        self.start = utils::verify_history(self, history)?;
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

    fn solve(&self, mode: IOMode, method: Solution) -> Result<(), NovaError> {
        match (self.players, method) {
            (2, Solution::Strong) => {
                <Self as acyclic::DynamicSolver<2, State>>::solve(&self, mode)
            },
            (10, Solution::Strong) => {
                <Self as acyclic::DynamicSolver<10, State>>::solve(&self, mode)
            },
            _ => {
                return Err(NovaError::SolverNotFound {
                    input_game_name: NAME,
                })
            },
        }
        Ok(())
    }
}

/* TRAVERSAL DECLARATIONS */

impl DynamicAutomaton<State> for Session {
    fn start(&self) -> State {
        self.start
    }

    fn transition(&self, state: State) -> Vec<State> {
        let (state, turn) = utils::unpack_turn(state, self.players);
        let mut next = self
            .by
            .iter()
            .map(|&choice| if state <= choice { state } else { choice })
            .map(|choice| {
                utils::pack_turn(
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
}

/* SUPPLEMENTAL DECLARATIONS */

impl Legible<State> for Session {
    fn decode(&self, string: String) -> Result<State, NovaError> {
        Ok(parse_state(&self, string)?)
    }

    fn encode(&self, state: State) -> String {
        let (elements, turn) = utils::unpack_turn(state, self.players);
        format!("{}-{}", elements, turn)
    }
}

/* SOLVING DECLARATIONS */

implement! { for Session =>
    Acyclic<2>,
    Acyclic<10>
}

impl Solvable<2> for Session {
    fn utility(&self, state: State) -> SVector<Utility, 2> {
        let (state, turn) = utils::unpack_turn(state, 2);
        let mut result = SVector::<Utility, 2>::zeros();
        result.fill(-1);
        result[turn] = 1;
        result
    }

    fn turn(&self, state: State) -> Turn {
        utils::unpack_turn(state, 2).1
    }
}

impl Solvable<10> for Session {
    fn utility(&self, state: State) -> SVector<Utility, 10> {
        let (state, turn) = utils::unpack_turn(state, 10);
        let mut result = SVector::<Utility, 10>::zeros();
        result.fill(-1);
        result[turn] = 9;
        result
    }

    fn turn(&self, state: State) -> Turn {
        utils::unpack_turn(state, 10).1
    }
}
