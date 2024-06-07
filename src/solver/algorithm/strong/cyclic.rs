//! # Strong Cyclic Solving Module
//!
//! This module implements strong cyclic solvers for all applicable types of
//! games through blanket implementations of the `acyclic::Solver` trait,
//! optimizing for specific game characteristics wherever possible.
//!
//! #### Authorship
//! - Max Fierro, 12/3/2023 (maxfierro@berkeley.edu)
//! - Ishir Garg, 3/12/2024 (ishirgarg@berkeley.edu)

use anyhow::{bail, Context, Result};

use std::collections::{HashMap, VecDeque};

use crate::database::KVStore;
use crate::game::{Bounded, Transition};
use crate::interface::IOMode;
use crate::model::game::State;
use crate::model::solver::SUtility;
use crate::solver::error::SolverError;
use crate::solver::record::sur::RecordBuffer;
use crate::solver::{Sequential, SimpleUtility};
use crate::util::Identify;

/* SOLVER */

pub fn solver<const B: usize, G>(game: &G, mode: IOMode) -> Result<()>
where
    G: Transition<B>
        + Bounded<B>
        + SimpleUtility<2, B>
        + Sequential<2, B>
        + Identify,
{
    todo!()
}

/* DATABASE INITIALIZATION */

/// TODO

/* SOLVING ALGORITHM */

fn cyclic_solver<const B: usize, G, D>(game: &G, db: &mut D) -> Result<()>
where
    G: Transition<B> + Bounded<B> + SimpleUtility<2, B> + Sequential<2, B>,
    D: KVStore,
{
    let mut winning_frontier = VecDeque::new();
    let mut tying_frontier = VecDeque::new();
    let mut losing_frontier = VecDeque::new();
    let mut child_counts = HashMap::new();
    enqueue_children(
        &mut child_counts,
        &mut winning_frontier,
        &mut losing_frontier,
        &mut tying_frontier,
        game.start(),
        game,
        db,
    )?;

    while !winning_frontier.is_empty()
        && !losing_frontier.is_empty()
        && !tying_frontier.is_empty()
    {
        let child = if !losing_frontier.is_empty() {
            losing_frontier
                .pop_front()
                .unwrap()
        } else if !winning_frontier.is_empty() {
            winning_frontier
                .pop_front()
                .unwrap()
        } else {
            tying_frontier.pop_front().unwrap()
        };

        let db_entry = RecordBuffer::from(db.get(&child).unwrap())
            .context("Failed to create record for middle state.")?;

        let child_utility = db_entry
            .get_utility(game.turn(child))
            .context("Failed to get utility from record.")?;

        let child_remoteness = db_entry.get_remoteness();
        let parents = game.retrograde(child);

        match child_utility {
            SUtility::Lose => {
                for parent in parents {
                    if *child_counts.get(&parent).unwrap() > 0 {
                        let mut buf = RecordBuffer::new(game.players())
                            .context(
                                "Failed to create record for end state.",
                            )?;

                        buf.set_utility([SUtility::Win, SUtility::Lose])?;

                        buf.set_remoteness(child_remoteness + 1)?;
                        db.put(&parent, &buf);

                        child_counts.insert(parent, 0);
                        winning_frontier.push_back(parent);
                    }
                }
            },
            SUtility::Tie => {
                for parent in parents {
                    let child_count = *child_counts.get(&parent).unwrap();
                    if child_count == 0 {
                        continue;
                    }

                    let mut buf = RecordBuffer::new(game.players())
                        .context("Failed to create record for end state.")?;

                    buf.set_utility([SUtility::Tie, SUtility::Tie])?;
                    buf.set_remoteness(child_remoteness + 1)?;
                    db.put(&parent, &buf);

                    tying_frontier.push_back(parent);
                    child_counts.insert(parent, 0);
                }
            },
            SUtility::Win => {
                for parent in parents {
                    let child_count = *child_counts.get(&parent).unwrap();
                    if child_count == 0 {
                        continue;
                    }

                    if child_count == 1 {
                        let mut buf = RecordBuffer::new(game.players())
                            .context(
                                "Failed to create record for end state.",
                            )?;

                        buf.set_utility([SUtility::Lose, SUtility::Win])?;

                        buf.set_remoteness(child_remoteness + 1)?;
                        db.put(&parent, &buf);

                        losing_frontier.push_back(parent);
                    }

                    child_counts.insert(parent, child_count - 1);
                }
            },
            SUtility::Draw => bail!(SolverError::SolverViolation {
                name: todo!(),
                hint: todo!(),
            }),
        }
    }

    for (parent, child_count) in child_counts {
        if child_count > 0 {
            let mut buf = RecordBuffer::new(game.players())
                .context("Failed to create record for end state.")?;

            buf.set_utility([SUtility::Draw, SUtility::Draw])?;
            db.put(&parent, &buf);
        }
    }

    Ok(())
}

fn enqueue_children<const B: usize, G, D>(
    child_counts: &mut HashMap<State<B>, usize>,
    winning_frontier: &mut VecDeque<State<B>>,
    losing_frontier: &mut VecDeque<State<B>>,
    tying_frontier: &mut VecDeque<State<B>>,
    curr_state: State<B>,
    game: &G,
    db: &mut D,
) -> Result<()>
where
    G: Transition<B> + Bounded<B> + SimpleUtility<2, B> + Sequential<2, B>,
    D: KVStore,
{
    if game.end(curr_state) {
        let mut buf = RecordBuffer::new(game.players())
            .context("Failed to create placeholder record.")?;
        buf.set_utility(game.utility(curr_state))
            .context("Failed to copy utility values to record.")?;
        buf.set_remoteness(0)
            .context("Failed to set remoteness for end state.")?;
        db.put(&curr_state, &buf);

        match game
            .utility(curr_state)
            .get(game.turn(curr_state))
            .unwrap()
        {
            SUtility::Win => winning_frontier.push_back(curr_state),
            SUtility::Tie => tying_frontier.push_back(curr_state),
            SUtility::Lose => losing_frontier.push_back(curr_state),
            SUtility::Draw => bail!(SolverError::SolverViolation {
                name: todo!(),
                hint: todo!(),
            }),
        }
        return Ok(());
    }

    let children = game.prograde(curr_state);
    child_counts.insert(curr_state, children.len());

    for child in children {
        if child_counts.contains_key(&child) {
            continue;
        }

        /// TODO: No recursion allowed
        enqueue_children(
            child_counts,
            winning_frontier,
            losing_frontier,
            tying_frontier,
            child,
            game,
            db,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    // TODO
}
