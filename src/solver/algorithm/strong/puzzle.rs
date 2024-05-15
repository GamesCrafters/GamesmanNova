//! # Strong Puzzle Solving Module
//!
//! This module implements routines for strongly solving puzzles.
//!
//! ### Authorship
//! - Ishir Garg (ishirgarg@berkeley.edu)

use anyhow::{Context, Result};

use std::collections::VecDeque;

use crate::database::KVStore;
use crate::game::{Bounded, Transition};
use crate::interface::IOMode;
use crate::model::game::State;
use crate::model::solver::{Remoteness, SUtility};
use crate::solver::record::surcc::{ChildCount, RecordBuffer};
use crate::solver::{ClassicPuzzle, SimpleUtility};

/* SOLVER */

pub fn dynamic_solver<const B: usize, G>(game: &G, mode: IOMode) -> Result<()>
where
    G: Transition<B> + Bounded<B> + ClassicPuzzle<B>,
{
    todo!()
}

/* DATABASE INITIALIZATION */

// TODO

/* SOLVING ALGORITHM */

/// Runs BFS starting from the ending primitive positions of a game, working its
/// way up the game tree in reverse. Assigns a remoteness and simple utiliity to
/// every winning and losing position. Draws (positions where winning is
/// impossible, but it is possible to play forever without losing) not assigned
/// a remoteness. This implementation uses the SURCC record to store child count
/// along with utility and remoteness.
fn reverse_bfs_solver<const B: usize, G, D>(db: &mut D, game: &G) -> Result<()>
where
    G: Transition<B> + Bounded<B> + ClassicPuzzle<B>,
    D: KVStore,
{
    let end_states = discover_child_counts(db, game)?;

    let mut winning_queue: VecDeque<State<B>> = VecDeque::new();
    let mut losing_queue: VecDeque<State<B>> = VecDeque::new();
    for end_state in end_states {
        let utility = game.utility(end_state);
        match utility {
            SUtility::Win => winning_queue.push_back(end_state),
            SUtility::Lose => losing_queue.push_back(end_state),
            SUtility::Tie => todo!(),
            SUtility::Draw => todo!(),
        };
        update_db_record(db, end_state, utility, 0, 0)?;
    }

    reverse_bfs_winning_states(db, game, &mut winning_queue)?;
    reverse_bfs_losing_states(db, game, &mut losing_queue)?;

    Ok(())
}

/// Performs BFS on winning states, marking visited states as a win
fn reverse_bfs_winning_states<const B: usize, G, D>(
    db: &mut D,
    game: &G,
    winning_queue: &mut VecDeque<State<B>>,
) -> Result<()>
where
    G: Transition<B> + Bounded<B>,
    D: KVStore,
{
    while let Some(state) = winning_queue.pop_front() {
        let buf = RecordBuffer::from(db.get(&state).unwrap())?;
        let child_remoteness = buf.get_remoteness();

        for parent in game.retrograde(state) {
            let child_count =
                RecordBuffer::from(db.get(&parent).unwrap())?.get_child_count();
            if child_count > 0 {
                winning_queue.push_back(parent);
                update_db_record(
                    db,
                    parent,
                    SUtility::Win,
                    1 + child_remoteness,
                    0,
                )?;
            }
        }
    }

    Ok(())
}

/// Performs BFS on losing states, marking visited states as a loss. Remoteness
/// is the shortest path to a primitive losing position.
fn reverse_bfs_losing_states<const B: usize, G, D>(
    db: &mut D,
    game: &G,
    losing_queue: &mut VecDeque<State<B>>,
) -> Result<()>
where
    G: Transition<B> + Bounded<B>,
    D: KVStore,
{
    while let Some(state) = losing_queue.pop_front() {
        let parents = game.retrograde(state);
        let child_remoteness =
            RecordBuffer::from(db.get(&state).unwrap())?.get_remoteness();

        for parent in parents {
            let child_count =
                RecordBuffer::from(db.get(&parent).unwrap())?.get_child_count();
            if child_count > 0 {
                // Update child count
                let mut buf = RecordBuffer::from(db.get(&parent).unwrap())
                    .context("Failed to get record for middle state")?;
                let new_child_count = buf.get_child_count() - 1;
                buf.set_child_count(new_child_count)?;
                db.put(&parent, &buf);

                // If all children have been solved, set this state as a losing
                // state
                if new_child_count == 0 {
                    losing_queue.push_back(parent);
                    update_db_record(
                        db,
                        parent,
                        SUtility::Lose,
                        1 + child_remoteness,
                        0,
                    )?;
                }
            }
        }
    }

    Ok(())
}

/// Updates the database record for a puzzle with given simple utility,
/// remoteness, and child count
fn update_db_record<const B: usize, D>(
    db: &mut D,
    state: State<B>,
    utility: SUtility,
    remoteness: Remoteness,
    child_count: ChildCount,
) -> Result<()>
where
    D: KVStore,
{
    let mut buf = RecordBuffer::from(db.get(&state).unwrap())
        .context("Failed to create record for middle state")?;
    buf.set_utility([utility])
        .context("Failed to set utility for state.")?;
    buf.set_remoteness(remoteness)
        .context("Failed to set remoteness for state.")?;
    buf.set_child_count(child_count)
        .context("Failed to set child count for state.")?;
    db.put(&state, &buf);

    Ok(())
}

fn discover_child_counts<const B: usize, G, D>(
    db: &mut D,
    game: &G,
) -> Result<Vec<State<B>>>
where
    G: Transition<B> + Bounded<B>,
    D: KVStore,
{
    let mut end_states = Vec::new();
    discover_child_counts_from_state(db, game, game.start(), &mut end_states)?;

    Ok(end_states)
}

fn discover_child_counts_from_state<const B: usize, G, D>(
    db: &mut D,
    game: &G,
    state: State<B>,
    end_states: &mut Vec<State<B>>,
) -> Result<()>
where
    G: Transition<B> + Bounded<B>,
    D: KVStore,
{
    let child_count = game.prograde(state).len() as ChildCount;

    if child_count == 0 {
        end_states.push(state);
    }

    let mut buf =
        RecordBuffer::new(1).context("Failed to create record for state")?;
    buf.set_utility([SUtility::Draw])
        .context("Failed to set remoteness for state")?;
    buf.set_child_count(child_count)
        .context("Failed to set child count for state.")?;
    db.put(&state, &buf);

    for &child in game
        .prograde(state)
        .iter()
        .chain(game.retrograde(state).iter())
    {
        if db.get(&child).is_none() {
            discover_child_counts_from_state(db, game, child, end_states)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    // TODO
}
