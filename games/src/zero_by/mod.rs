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
    solvers::{acyclic::AcyclicSolver, cyclic::CyclicSolve, tiered::TierSolve},
    solvers::{AcyclicallySolvable, CyclicallySolvable, Solvable, TierSolvable},
    State, Value, Variant,
};
use std::collections::HashSet;

implement! { for Session =>
    AcyclicGame,
    TierSolvable,
    AcyclicallySolvable,
    CyclicallySolvable,
    SmallGame
}

/* GAME IMPLEMENTATION */

const DEFAULT_STARTING_COINS: State = 1000;

/// Represents a 10-to-0-by-1-or-2 game instance.
pub struct Session {
    starting_coins: State,
}

impl Game for Session {
    fn initialize(variant: Option<Variant>) -> Self {
        if let Some(num) = variant {
            Session {
                starting_coins: num.parse::<u64>().unwrap(),
            }
        } else {
            Session {
                starting_coins: DEFAULT_STARTING_COINS,
            }
        }
    }

    fn default(&self) -> State {
        self.starting_coins
    }

    fn adjacent(&self, state: State) -> HashSet<State> {
        let mut children = HashSet::new();
        if state >= 2 {
            children.insert(state - 2);
        }
        if state >= 1 {
            children.insert(state - 1);
        }
        children
    }

    fn value(&self, state: State) -> Option<Value> {
        if state > 0 {
            None
        } else {
            Some(Value::Lose(0))
        }
    }

    fn id(&self) -> String {
        format!("zero-by.{}", self.starting_coins)
    }
}

impl Solvable for Session {
    fn solvers(&self) -> Vec<(Option<String>, fn(&Self, bool, bool) -> Value)> {
        vec![
            (None, Self::acyclic_solve),
            (Some(Self::acyclic_solver_name()), Self::acyclic_solve),
            (Some(Self::cyclic_solver_name()), Self::cyclic_solve),
            (Some(Self::tier_solver_name()), Self::tier_solve),
        ]
    }
}
