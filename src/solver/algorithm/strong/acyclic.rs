//! # Strong Acyclic Solving Module
//!
//! This module implements strong acyclic solving routines.
//!
//! #### Authorship
//! - Max Fierro, 12/3/2023 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};

use crate::database::volatile;
use crate::database::{KVStore, Tabular};
use crate::game::{Bounded, Game, Transition};
use crate::interface::IOMode;
use crate::model::game::PlayerCount;
use crate::model::solver::{IUtility, Remoteness};
use crate::solver::record::mur::RecordBuffer;
use crate::solver::{Extensive, IntegerUtility, RecordType};

/* SOLVERS */

pub fn solver<const N: PlayerCount, const B: usize, G>(
    game: &G,
    mode: IOMode,
) -> Result<()>
where
    G: Transition<B>
        + Bounded<B>
        + IntegerUtility<N, B>
        + Extensive<N, B>
        + Game,
{
    let db = volatile_database(game)
        .context("Failed to initialize volatile database.")?;

    let table = db
        .select_table(game.id())
        .context("Failed to select solution set database table.")?;

    backward_induction(table, game)
        .context("Failed solving algorithm execution.")?;

    Ok(())
}

/* DATABASE INITIALIZATION */

/// Initializes a volatile database, creating a table schema according to the
/// solver record layout, initializing a table with that schema, and switching
/// to that table before returning the database handle.
fn volatile_database<const N: usize, const B: usize, G>(
    game: &G,
) -> Result<volatile::Database>
where
    G: Extensive<N, B> + Game,
{
    let id = game.id();
    let db = volatile::Database::initialize();

    let schema = RecordType::MUR(N)
        .try_into()
        .context("Failed to create table schema for solver records.")?;

    db.create_table(id, schema)
        .context("Failed to create database table for solution set.")?;

    Ok(db)
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
    G: Transition<B> + Bounded<B> + IntegerUtility<N, B> + Extensive<N, B>,
{
    let mut stack = Vec::new();
    stack.push(game.start());

    while let Some(curr) = stack.pop() {
        let children = game.prograde(curr);
        let mut buf = RecordBuffer::new(game.players())
            .context("Failed to create placeholder record.")?;

        if db.get(&curr).is_none() {
            db.put(&curr, &buf)?;

            if game.end(curr) {
                buf = RecordBuffer::new(game.players())
                    .context("Failed to create record for end state.")?;

                buf.set_utility(game.utility(curr))
                    .context("Failed to copy utility values to record.")?;

                buf.set_remoteness(0)
                    .context("Failed to set remoteness for end state.")?;

                db.put(&curr, &buf)?;
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

            db.put(&curr, &optimal)?;
        }
    }
    Ok(())
}
