//! # Acyclic Solver Module
//!
//! This module implements an acyclic solver for all applicable types of games
//! through a blanket implementation of the `AcyclicallySolvable` trait.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use super::{choose_value, AcyclicallySolvable};
use crate::archetypes::Game;
use crate::{State, Value};
use std::collections::{HashMap, HashSet};

/* SOLVER NAME */

/// Defines this solver's name for GamesmanNova's interfaces.
static SOLVER_NAME: &str = "acyclic";

/* COMFORTER IMPLEMENTATION */

/// Indicates that a game has the capacity to perform an acyclic solve on itself.
pub trait AcyclicSolve {
    /// Returns the value of an arbitrary state of the game.
    fn acyclic_solve(&self) -> Value;
    /// Returns the name of this solver type.
    fn acyclic_solver_name(&self) -> &'static str;
}

/// Blanket implementation of the acyclic solver for all acyclically solvable games.
impl<G: AcyclicallySolvable> AcyclicSolve for G {
    fn acyclic_solve(&self) -> Value {
        let default_entry = self.state();
        let mut seen: HashMap<State, Value> = HashMap::new();
        traverse(default_entry, self, &mut seen)
    }

    fn acyclic_solver_name(&self) -> &'static str {
        SOLVER_NAME
    }
}

/* HELPER FUNCTIONS */

/// Recursive algorithm for traversing a game with DAG-structured states and
/// returning the value of the entry point.
fn traverse<G>(state: State, game: &G, seen: &mut HashMap<State, Value>) -> Value
where
    G: Game,
    G: AcyclicallySolvable,
{
    if let Some(out) = game.value(state) {
        return out;
    }
    let mut available: HashSet<Value> = HashSet::new();
    for state in game.children(state) {
        if let Some(out) = seen.get(&state).copied() {
            available.insert(out);
        } else {
            let out = traverse(state, game, seen);
            available.insert(out);
            seen.insert(state, out);
        }
    }
    choose_value(available)
}
