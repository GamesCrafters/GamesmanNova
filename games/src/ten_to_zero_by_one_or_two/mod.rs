//! # 10-to-0-by-1-or-2 Game Module
//!
//! 10-to-0-by-1-or-2 is a small acyclic game.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::implement;
use std::collections::HashSet;
use core::{
    archetypes::{AcyclicGame, Game, SmallGame},
    solvers::{acyclic::AcyclicSolve, cyclic::CyclicSolve, tiered::TierSolve},
    solvers::{AcyclicallySolvable, CyclicallySolvable, TierSolvable},
    State, Value,
};

implement! { for Session =>
    AcyclicGame,
    TierSolvable,
    CyclicallySolvable,
    AcyclicallySolvable,
    SmallGame
}

const STARTING_COINS: u64 = 1000;

/// Represents a 10-to-0-by-1-or-2 game instance.
pub struct Session;

impl Session {
    /// Spawns a new game session.
    pub fn new() -> Self { Session {} }
}

impl Game for Session {
    fn state(&self) -> State {
        State(STARTING_COINS)
    }
    fn children(&self, state: State) -> HashSet<State> {
        let mut children = HashSet::new();
        if state.0 >= 2 {
            children.insert(State(state.0 - 2));
        }
        if state.0 >= 1 {
            children.insert(State(state.0 - 1));
        }
        children
    }
    fn value(&self, state: State) -> Option<Value> {
        if state.0 > 0 {
            None
        } else {
            Some(Value::Lose(0))
        }
    }
    fn solvers(&self) -> Vec<(&'static str, fn(&Self) -> Value)> {
        vec![
            (self.acyclic_solver_name(), Self::acyclic_solve),
            (self.cyclic_solver_name(), Self::cyclic_solve),
            (self.tier_solver_name(), Self::tier_solve),
        ]
    }
}
