//! # Acyclic Solver Module
//!
//! This module implements an acyclic solver for all applicable types of games
//! through a blanket implementation of the `AcyclicallySolvable` trait.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use nalgebra::{SMatrix, SVector};

use crate::databases::{bpdb::BPDatabase, Database};
use crate::games::AcyclicallySolvable;
use crate::interfaces::terminal::cli::IOMode;
use crate::models::{Record, State};
use std::collections::HashSet;

/* SOLVER NAME */

/// Defines this solver's name for GamesmanNova's interfaces.
const SOLVER_NAME: &str = "recursive-acyclic";

/* COMFORTER IMPLEMENTATION */

pub trait AcyclicSolver<const N: usize>
{
    fn solve(game: &Self, mode: Option<IOMode>) -> Record<N>;

    fn name() -> String;
}

impl<G, const N: usize> AcyclicSolver<N> for G
where
    G: AcyclicallySolvable<N>,
{
    fn solve(game: &G, mode: Option<IOMode>) -> Record<N>
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

fn traverse<G: AcyclicallySolvable<N>, const N: usize>(
    state: State,
    game: &G,
    db: &mut BPDatabase<N>,
) -> Record<N>
{
    if game.accepts(state) {
        return Record::default().with_util(
            game.utility(state).expect(&format!(
                "No utility vector defined for state {}",
                state
            )),
        )
    }
    let mut available: HashSet<Record<N>> = HashSet::new();
    for state in game.transition(state) {
        if let Some(out) = db.get(state) {
            available.insert(out);
        } else {
            let out = traverse(state, game, db);
            available.insert(out);
            db.put(state, out);
        }
    }
    let matrix = game.weights();
    let coalition = game.coalesce(state);
    let record = select_record(matrix, coalition, available);
    db.put(state, record);
    record
}

/// Selects the record in `available` that maximizes the dot product between the
/// coalition vector associated with the game state and the record's utility
/// vector, after it is multiplied by the state's utility matrix. If multiple
/// records are at a maximum, selects the record with the lowest remoteness.
///
/// All of the operations in this function should happen on the stack due to
/// statically-sized types and in-place modification.
fn select_record<const N: usize>(
    matrix: SMatrix<i64, N, N>,
    coalition: SVector<i64, N>,
    available: HashSet<Record<N>>,
) -> Record<N>
{
    let mut dot = i64::MIN;
    let mut rem = u64::MAX;
    let mut result = Record::default();
    for r in available.into_iter() {
        let curr_dot = (matrix * r.util).dot(&coalition);
        if curr_dot > dot || (curr_dot == dot && rem > r.rem) {
            result.util = r.util;
            result.rem = rem + 1;
            dot = curr_dot;
            rem = r.rem;
        }
    }
    result
}
