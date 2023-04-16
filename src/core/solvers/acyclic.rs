//! # Acyclic Solver Module
//!
//! This module implements an acyclic solver for all applicable types of games
//! through a blanket implementation of the `AcyclicallySolvable` trait.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use super::{choose_value, AcyclicallySolvable};
use crate::games::archetypes::Game;
use crate::core::databases::{bpdb::BPDatabase, Database};
use crate::core::{State, Value};
use std::collections::HashSet;

/* SOLVER NAME */

/// Defines this solver's name for GamesmanNova's interfaces.
const SOLVER_NAME: &str = "acyclic";

/* COMFORTER IMPLEMENTATION */

/// Indicates that a game could theoretically be solved acyclically.
pub trait AcyclicSolver {
    /// Returns the value of an arbitrary state of the game, and uses `read` 
    /// and `write` for specifying I/O preferences to database implementations.
    fn acyclic_solve(game: &Self, read: bool, write: bool) -> Value;
    /// Returns the name of this solver type.
    fn acyclic_solver_name() -> String;
}

/// Blanket implementation of the acyclic solver for all acyclically solvable 
/// games.
impl<G: AcyclicallySolvable> AcyclicSolver for G {
    fn acyclic_solve(game: &Self, read: bool, write: bool) -> Value {
        let state = game.start();
        let mut db = BPDatabase::new(game.id(), read, write);
        traverse(state, game, &mut db)
    }

    fn acyclic_solver_name() -> String {
        SOLVER_NAME.to_owned()
    }
}

/* HELPER FUNCTIONS */

/// Recursive algorithm for traversing a game with DAG-structured states and
/// returning the value of the entry point.
fn traverse<G>(state: State, game: &G, db: &mut BPDatabase) -> Value
where
    G: Game,
    G: AcyclicallySolvable,
{
    if let Some(out) = game.value(state) {
        return out;
    }
    let mut available: HashSet<Value> = HashSet::new();
    for state in game.adjacent(state) {
        if let Some(out) = db.get(state).clone() {
            available.insert(out);
        } else {
            let out = traverse(state, game, db);
            available.insert(out);
            db.insert(state, Some(out));
        }
    }
    let value = choose_value(available);
    db.insert(state, Some(value));
    value
}
