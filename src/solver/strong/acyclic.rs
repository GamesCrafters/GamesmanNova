//! # Strong Acyclic Solving Module
//!
//! This module implements strong acyclic solvers for all applicable types of
//! games through blanket implementations of the `acyclic::Solver` trait,
//! optimizing for specific game characteristics wherever possible.
//!
//! #### Authorship
//!
//! - Max Fierro, 12/3/2023 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};

use crate::database::engine::volatile;
use crate::database::object::schema::{Attribute, Datatype, SchemaBuilder};
use crate::database::{KVStore, Tabular};
use crate::game::{Acyclic, Bounded, DTransition, STransition, Solvable};
use crate::interface::IOMode;
use crate::model::{PlayerCount, State};
use crate::schema;
use crate::solver::MAX_TRANSITIONS;

/* SOLVERS */

pub fn dynamic_solver<const N: usize, G>(game: &G, mode: IOMode) -> Result<()>
where
    G: Acyclic<N> + DTransition<State> + Bounded<State> + Solvable<N>,
{
    let mut db = volatile_database(&game.id())
        .context("Failed to initialize database.")?;

    dynamic_backward_induction(&mut db, game);
    Ok(())
}

pub fn static_solver<const N: usize, G>(game: &G, mode: IOMode) -> Result<()>
where
    G: Acyclic<N>
        + STransition<State, MAX_TRANSITIONS>
        + Bounded<State>
        + Solvable<N>,
{
    let mut db = volatile_database(&game.id())
        .context("Failed to initialize database.")?;

    static_backward_induction(&mut db, game);
    Ok(())
}

/* SOLVING ALGORITHMS */

fn dynamic_backward_induction<const N: PlayerCount, G, D>(db: &mut D, game: &G)
where
    G: Acyclic<N> + Bounded<State> + DTransition<State> + Solvable<N>,
    D: KVStore,
{
    let mut stack = Vec::new();
    stack.push(game.start());
    while let Some(curr) = stack.pop() {
        let children = game.prograde(curr);
        if let None = db.get(curr) {
            db.put(curr, Record::default());
            if children.is_empty() {
                let record = Record::default()
                    .with_utility(game.utility(curr))
                    .with_remoteness(0);
                db.put(curr, record)
            } else {
                stack.push(curr);
                stack.extend(
                    children
                        .iter()
                        .filter(|&x| db.get(*x).is_none()),
                );
            }
        } else {
            db.put(
                curr,
                children
                    .iter()
                    .map(|&x| db.get(x).unwrap())
                    .max_by(|r1, r2| r1.cmp(&r2, game.turn(curr)))
                    .unwrap(),
            );
        }
    }
}

fn static_backward_induction<const N: PlayerCount, G, D>(db: &mut D, game: &G)
where
    G: Acyclic<N>
        + STransition<State, MAX_TRANSITIONS>
        + Bounded<State>
        + Solvable<N>,
    D: KVStore,
{
    let mut stack = Vec::new();
    stack.push(game.start());
    while let Some(curr) = stack.pop() {
        let children = game.prograde(curr);
        if let None = db.get(curr) {
            db.put(curr, Record::default());
            if children
                .iter()
                .all(|x| x.is_none())
            {
                let record = Record::default()
                    .with_utility(game.utility(curr))
                    .with_remoteness(0);
                db.put(curr, record)
            } else {
                stack.push(curr);
                stack.extend(
                    children
                        .iter()
                        .filter_map(|&x| x)
                        .filter(|&x| db.get(x).is_none()),
                );
            }
        } else {
            db.put(
                curr,
                children
                    .iter()
                    .filter_map(|&x| x)
                    .map(|x| db.get(x).unwrap())
                    .max_by(|r1, r2| r1.cmp(&r2, game.turn(curr)))
                    .unwrap(),
            );
        }
    }
}

/* HELPERS */

fn volatile_database(table: &str) -> Result<volatile::Database> {
    let schema = schema! {
        "Player 1 Utility"; Datatype::SINT; 8,
        "Player 2 Utility"; Datatype::SINT; 8,
        "Total Remoteness"; Datatype::UINT; 8
    };

    let mut db = volatile::Database::initialize();
    db.create_table(table, schema);
    db.select_table(table);

    Ok(db)
}
