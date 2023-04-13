//! # 10-to-0-by-1-or-2 Game Module
//!
//! 10-to-0-by-1-or-2 is a small acyclic game.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

/* INFRA IMPORTS */

use crate::implement;
use core::{
    archetypes::{AcyclicGame, Game, SmallGame},
    solvers::{acyclic::AcyclicSolve, cyclic::CyclicSolve, tiered::TierSolve},
    solvers::{AcyclicallySolvable, CyclicallySolvable, TierSolvable},
    State, Value,
};
use std::collections::{HashMap, HashSet};

implement! { for Session =>
    AcyclicGame,
    TierSolvable,
    CyclicallySolvable,
    AcyclicallySolvable,
    SmallGame
}

/* GAME IMPLEMENTATION */

const STARTING_COINS: State = 1000;

/// Represents a 10-to-0-by-1-or-2 game instance.
pub struct Session {
    coins: State,
}

impl Session {
    /// Spawns a new 10-to-0-by-1-or-2 game session.
    pub fn new() -> Self {
        Session {
            coins: STARTING_COINS,
        }
    }
}

impl Game for Session {
    fn state(&self) -> State {
        self.coins
    }

    fn children(state: State) -> HashSet<State> {
        let mut children = HashSet::new();
        if state >= 2 {
            children.insert(state - 2);
        }
        if state >= 1 {
            children.insert(state - 1);
        }
        children
    }

    fn value(state: State) -> Option<Value> {
        if state > 0 {
            None
        } else {
            Some(Value::Lose(0))
        }
    }

    fn solvers(&self) -> Vec<(Option<&str>, fn(&Self) -> Value)> {
        vec![
            (None, Self::acyclic_solve),
            (Some(self.acyclic_solver_name()), Self::acyclic_solve),
            (Some(self.cyclic_solver_name()), Self::cyclic_solve),
            (Some(self.tier_solver_name()), Self::tier_solve),
        ]
    }
}
