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

use std::collections::HashMap;

use crate::{collection, implement};
use crate::games::{
    Game, GameData, Solvable, Automaton,
};
use crate::models::Utility;
use crate::solvers::acyclic::AcyclicSolver;
use crate::{models::{Solver, State, Variant, Player}, errors::VariantError};
use nalgebra::{Matrix2, SMatrix, SVector, Vector2};
use variants::*;

use super::AcyclicallySolvable;

/* SUBMODULES */

mod variants;
mod utils;

/* GAME DATA */

const NAME: &str = "Zero-By";
const AUTHOR: &str = "Max Fierro";
const CATEGORY: &str = "Combinatorial n-player zero-sum game";
const ABOUT: &str = 
"Many players take turns removing a number of elements from a set of arbitrary \
size. They can make a choice of how many elements to remove (and of how many \
elements to start out with) based on the game variant. The player who is left \
with 0 elements in their turn loses. A player cannot remove more elements than \
currently available in the set.";

/* GAME IMPLEMENTATION */

pub struct Session
{
    variant: Option<String>,
    players: Player,
    from: State,
    by: Vec<u64>,
}

impl Game for Session
{
    fn initialize(variant: Option<Variant>) -> Result<Self, VariantError>
    {
        if let Some(v) = variant {
            parse_variant(v)
        } else {
            parse_variant(VARIANT_DEFAULT.to_owned())
        }
    }

    fn id(&self) -> String
    {
        if let Some(variant) = self.variant.clone() {
            format!("{}.{}", NAME, variant)
        } else {
            NAME.to_owned()
        }
    }

    fn info(&self) -> GameData
    {
        GameData {
            name: NAME.to_owned(),
            author: AUTHOR.to_owned(),
            about: ABOUT.to_owned(),
            category: CATEGORY.to_owned(),
            variant_protocol: VARIANT_PROTOCOL.to_owned(),
            variant_pattern: VARIANT_PATTERN.to_owned(),
            variant_default: VARIANT_DEFAULT.to_owned(),
        }
    }

    fn solvers(&self) -> HashMap<String, Solver<Self>>
    {
        match self.players {
            2 => {
                let acyclic: Solver<Self> = <Self as AcyclicSolver<2>>::solve;
                collection! {
                    <Self as AcyclicSolver<2>>::name() => acyclic,
                }
            },
            10 => {
                let acyclic: Solver<Self> = <Self as AcyclicSolver<10>>::solve;
                collection! {
                    <Self as AcyclicSolver<10>>::name() => acyclic,
                }
            }
            _ => todo!()
        }
    }
}

impl Automaton<State> for Session
{
    fn start(&self) -> State
    {
        utils::pack_turn(self.from, 0, self.players)
    }

    fn transition(&self, state: State) -> Vec<State>
    {
        let (state, turn) = utils::unpack_turn(state, self.players);
        self.by
            .iter()
            .cloned()
            .map(|choice| if state <= choice { state } else { choice })
            .filter(|&choice| state >= choice)
            .map(|choice| state - choice)
            .map(|output| {
                utils::pack_turn(
                    output, 
                    (turn + 1) % self.players, 
                    self.players
                )
            })
            .collect::<Vec<State>>()
    }

    fn accepts(&self, state: State) -> bool
    {
        let (state, _) = utils::unpack_turn(state, self.players);
        state == 0
    }
}

/* SOLVABLE DECLARATIONS */

implement! { for Session =>
    AcyclicallySolvable<2>,
    AcyclicallySolvable<10>
}

impl Solvable<2> for Session
{
    fn weights(&self) -> SMatrix<Utility, 2, 2>
    {
        Matrix2::<Utility>::identity()
    }

    fn utility(&self, state: State) -> Option<SVector<Utility, 2>>
    {
        let (state, turn) = utils::unpack_turn(state, 2);
        if !self.accepts(state) {
            None
        } else if turn % 2 == 0 {
            Some(Vector2::new(-1, 1))
        } else {
            Some(Vector2::new(1, -1))
        }
    }

    fn coalesce(&self, state: State) -> SVector<Utility, 2> {
        let (_, turn) = utils::unpack_turn(state, 2);
        Vector2::new(
            ((turn + 1) % 2).into(), 
            (turn % 2).into()
        )
    }
}

impl Solvable<10> for Session
{
    fn weights(&self) -> SMatrix<Utility, 10, 10>
    {
        SMatrix::<Utility, 10, 10>::identity()
    }

    fn utility(&self, state: State) -> Option<SVector<Utility, 10>>
    {
        let (state, turn) = utils::unpack_turn(state, 10);
        if !self.accepts(state) {
            None
        } else {
            let mut result: SVector<Utility, 10> = SVector::<Utility, 10>::zeros();
            for i in 0..10 {
                if turn == i {
                    result[i as usize] = -9;
                } else {
                    result[i as usize] = 1;
                }
            }
            Some(result)
        }
    }

    fn coalesce(&self, state: State) -> SVector<Utility, 10> {
        let (_, turn) = utils::unpack_turn(state, 10);
        let mut result: SVector<Utility, 10> = SVector::<Utility, 10>::zeros();
        for i in 0..10 {
            if turn == i {
                result[i as usize] = 1;
            } else {
                result[i as usize] = 0;
            }
        }
        result
    }
}
