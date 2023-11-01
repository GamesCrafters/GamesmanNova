//! # Acyclic Solver Module
//!
//! This module implements an acyclic solver for all applicable types of games
//! through a blanket implementation of the `AcyclicallySolvable` trait.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::databases::{bpdb::BPDatabase, Database};
use crate::games::AcyclicallySolvable;
use crate::interfaces::terminal::cli::IOMode;
use crate::models::{Record, State};
use std::collections::HashSet;

/* SOLVER NAME */

/// Defines this solver's name for GamesmanNova's interfaces.
const SOLVER_NAME: &str = "recursive-acyclic";

/* COMFORTER IMPLEMENTATION */

/// Indicates that a game could theoretically be solved acyclically.
pub trait AcyclicSolver<const N: usize>
{
    /// Returns the value of an arbitrary state of the game, specifying I/O
    /// preferences to database implementations.
    fn solve(game: &Self, mode: Option<IOMode>) -> Record<N>;

    /// Returns the name of this solver type.
    fn name() -> String;
}

/// Blanket implementation of the acyclic solver for all acyclically solvable
/// games of two players.
impl<G: AcyclicallySolvable<2>> AcyclicSolver<2> for G
{
    fn solve(game: &Self, mode: Option<IOMode>) -> Record<2>
    {
        let mut db = BPDatabase::new(game.id(), mode);
        let state = game.start();
        traverse(state, game, &mut db)
    }

    fn name() -> String
    {
        SOLVER_NAME.to_owned()
    }
}

/* HELPER FUNCTIONS */

/// Recursive algorithm for traversing a game with DAG-structured states and
/// returning the value of the entry point.
fn traverse<G>(state: State, game: &G, db: &mut BPDatabase) -> Record<2>
where
    G: AcyclicallySolvable<2>,
{
    if let Some(out) = game.utility(state) {
        return out
    }
    let mut available: HashSet<Record> = HashSet::new();
    for state in game.transition(state) {
        if let Some(out) = db.get(state) {
            available.insert(out);
        } else {
            let out = traverse(state, game, db);
            available.insert(out);
            db.put(state, out);
        }
    }
    let value = choose_value(available);
    db.put(state, value);
    value
}
