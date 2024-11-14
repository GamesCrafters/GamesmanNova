//! # Strong Acyclic Solving Module
//!
//! This module implements strong acyclic solving routines.

use anyhow::{Context, Result};

use crate::database::record::mur::RecordBuffer;
use crate::database::volatile;
use crate::database::KVStore;
use crate::database::RecordType;
use crate::game::model::PlayerCount;
use crate::game::Information;
use crate::game::{Bounded, Transition};
use crate::interface::IOMode;
use crate::solver::model::{IUtility, Remoteness};
use crate::solver::{IntegerUtility, Sequential};
use crate::util::Identify;

/* SOLVERS */

pub fn solver<const N: PlayerCount, const B: usize, G>(
    game: &G,
    mode: IOMode,
) -> Result<()>
where
    G: Transition<B>
        + Bounded<B>
        + IntegerUtility<N, B>
        + Sequential<N, B>
        + Identify
        + Information,
{
    let table_name = format!("{}_solution", G::info().name);
    let db = volatile::Database::new()?;
    let table_schema = RecordType::MUR(N)
        .try_into_schema(&table_name)
        .context("Failed to create table schema for solver records.")?;

    match mode {
        IOMode::Constructive => (),
        IOMode::Overwrite => db
            .drop_resource(&table_name)
            .context("Failed to drop database table being overwritten.")?,
    }

    let solution_table = db
        .create_resource(table_schema)
        .context("Failed to create database table for solution set.")?;

    db.build_transaction()
        .writing(solution_table)
        .action(|mut working_set| {
            let mut t = working_set.get_writing(solution_table);
            backward_induction(&mut (*t), game)
                .context("Solving algorithm encountered an error.")
        })
        .execute()
        .context("Solving algorithm transaction failed.")
}

/* SOLVING ALGORITHMS */

/// Performs an iterative depth-first traversal of the game tree, assigning to
/// each game `state` a remoteness and utility values for each player within
/// `db`. This uses heap-allocated memory for keeping a stack of positions to
/// facilitate DFS, as well as for communicating state transitions.
fn backward_induction<const N: PlayerCount, const B: usize, D, G>(
    db: &mut D,
    game: &G,
) -> Result<()>
where
    D: KVStore,
    G: Transition<B> + Bounded<B> + IntegerUtility<N, B> + Sequential<N, B>,
{
    let mut stack = Vec::new();
    stack.push(game.start());
    while let Some(curr) = stack.pop() {
        let children = game.prograde(curr);
        let mut buf = RecordBuffer::new(game.players())
            .context("Failed to create placeholder record.")?;

        if db.get(&curr).is_none() {
            db.insert(&curr, &buf)?;
            if game.end(curr) {
                buf = RecordBuffer::new(game.players())
                    .context("Failed to create record for end state.")?;

                buf.set_utility(game.utility(curr))
                    .context("Failed to copy utility values to record.")?;

                buf.set_remoteness(0)
                    .context("Failed to set remoteness for end state.")?;

                db.insert(&curr, &buf)?;
            } else {
                stack.push(curr);
                stack.extend(
                    children
                        .iter()
                        .filter(|&x| db.get(x).is_none()),
                );
            }
        } else {
            let mut optimal = buf;
            let mut max_val = IUtility::MIN;
            let mut min_rem = Remoteness::MAX;
            for state in children {
                let buf = RecordBuffer::from(db.get(&state).unwrap())
                    .context("Failed to create record for middle state.")?;

                let val = buf
                    .get_utility(game.turn(state))
                    .context("Failed to get utility from record.")?;

                let rem = buf.get_remoteness();
                if val > max_val || (val == max_val && rem < min_rem) {
                    max_val = val;
                    min_rem = rem;
                    optimal = buf;
                }
            }

            optimal
                .set_remoteness(min_rem + 1)
                .context("Failed to set remoteness for solved record.")?;

            db.insert(&curr, &optimal)?;
        }
    }
    Ok(())
}
