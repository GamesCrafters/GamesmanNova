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

/* INFRA IMPORTS */

use super::{AcyclicGame, Game, SmallGame};
use crate::core::{
    solvers::{acyclic::AcyclicSolver, cyclic::CyclicSolver, tiered::TierSolver},
    solvers::{AcyclicallySolvable, CyclicallySolvable, Solvable, TierSolvable},
    State, Value, Variant,
};
use crate::implement;
use std::collections::HashSet;

implement! { for Session =>
    AcyclicGame,
    TierSolvable,
    AcyclicallySolvable,
    CyclicallySolvable,
    SmallGame
}

/* GAME IMPLEMENTATION */

const DEFAULT_FROM: State = 10;
const DEFAULT_BY: [u16; 2] = [1, 2];

/// Represents a Zero-By game instance.
pub struct Session {
    id: Option<String>,
    from: State,
    by: Vec<u16>
}

impl Game for Session {
    fn initialize(variant: Option<Variant>) -> Self {
        if let Some(num) = variant {
            // Turn variant string into FROM and BY options
            todo!()
        } else {
            Session {
                id: None,
                from: DEFAULT_FROM,
                by: DEFAULT_BY.to_vec()
            }
        }
    }

    fn start(&self) -> State {
        self.from
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
        if let Some(id) = self.id.clone() {
            format!("zero-by.{}", id)
        } else {
            "zero-by".to_owned()
        }
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
