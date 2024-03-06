//! # Strong Acyclic Solving Module
//!
//! This module implements strong acyclic solvers for all applicable types of
//! games through blanket implementations of the `acyclic::Solver` trait,
//! optimizing for specific game characteristics wherever possible.
//!
//! #### Authorship
//!
//! - Max Fierro, 12/3/2023 (maxfierro@berkeley.edu)

use crate::database::engine::volatile;
use crate::game::{Acyclic, Bounded, DTransition, STransition};
use crate::interface::IOMode;
use crate::model::{PlayerCount, State};
use crate::solver::MAX_TRANSITIONS;

/* SOLVERS */

pub fn dynamic<const N: usize, G>(game: &G, mode: IOMode)
where
    G: Acyclic<N> + DTransition<State> + Bounded<State>
{
    let mut db = volatile::Database::initialize();
    dynamic_backward_induction(db, game);
}

pub fn static<const N: usize, G>(game: &G, mode: IOMode)
where
    G: Acyclic<N> + STransition<State, MAX_TRANSITIONS> + Bounded<State>
{
    let mut db = volatile::Database::initialize();
    static_backward_induction(db, game);
}

/* SOLVING ALGORITHMS */

fn dynamic_backward_induction<const N: PlayerCount, G>(
    db: &mut BPDatabase<N>,
    game: &G,
) where
    G: Acyclic<N> + Bounded<State> + DTransition<State>,
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

fn static_backward_induction<const N: PlayerCount, G>(
    db: &mut BPDatabase<N>,
    game: &G,
) where
    G: Acyclic<N> + STransition<State, MAX_TRANSITIONS> + Bounded<State>,
{
    let mut stack = Vec::new();
    stack.push(game.start());
    while let Some(curr) = stack.pop() {
        let children = game.transition(curr);
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
