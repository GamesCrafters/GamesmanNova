//! # Strong Puzzle Solving Module
//!
//! This module implements routines for strongly solving puzzles.

use anyhow::{Context, Result};

use crate::database::volatile;
use crate::database::{Map, Tabular};
use crate::game::{Bounded, DTransition, GeneralSum, Playable, STransition};
use crate::interface::IOMode;
use crate::model::{PlayerCount, Remoteness, State, Utility};
use crate::solver::record::mur::RecordBuffer;
use crate::solver::{RecordType, MAX_TRANSITIONS};

pub fn dynamic_solver<const N: usize, G>(game: &G, mode: IOMode) -> Result<()>
where
    G: DTransition<State> + Bounded<State> + Playable<N> + GeneralSum<N>,
{
    let mut db = volatile_database(game)
        .context("Failed to initialize volatile database.")?;

    bfs(&mut db, game).context("Failed solving algorithm execution.")?;

    Ok(())
}

fn bfs<G, D>(game: &G, db: &mut D)
where
    G: DTransition<State> + Bounded<State> + SimpleSum<N>,
    D: Map,
{
    let end_states = discover_end_states_helper(db, game);

    for state in end_states {
        let mut buf = RecordBuffer::new()
            .context("Failed to create placeholder record.")?;
        buf.set_remoteness(0)
            .context("Failed to set remoteness for end state.")?;
        db.put(state, &buf);

        bfs_state(db, game, state);
    }
}

fn bfs_state<G, D>(db: &mut D, game: &G)
where
    G: DTransition<State> + Bounded<State> + SimpleSum<N>,
    D: Map,
{
}

fn discover_end_states<G, D>(db: &mut D, game: &G) -> Vec<State>
where
    G: DTransition<State> + Bounded<State> + SimpleSum<N>,
    D: Map,
{
    let visited = HashSet::new();
    let end_states = Vec::new();

    discover_end_states(db, game, game.start(), visited, end_states);

    end_states
}

fn discover_end_states_helper<G, D>(
    db: &mut D,
    game: &G,
    state: State,
    visited: HashSet<State>,
    end_states: Vec<State>,
) where
    G: DTransition<State> + Bounded<State> + SimpleSum<N>,
    D: Map,
{
    visited.insert(state);

    if game.end(state) {
        end_states.insert(state);
    }

    for child in game.prograde(state) {
        if !visted.contains(child) {
            discover_end_states(db, game, child, visited, end_states);
        }
    }
}
/* DATABASE INITIALIZATION */

/// Initializes a volatile database, creating a table schema according to the
/// solver record layout, initializing a table with that schema, and switching
/// to that table before returning the database handle.
fn volatile_database<const N: usize, G>(game: &G) -> Result<volatile::Database>
where
    G: Playable<N>,
{
    let id = game.id();
    let db = volatile::Database::initialize();

    let schema = RecordType::REMOTE(N)
        .try_into()
        .context("Failed to create table schema for solver records.")?;
    db.create_table(&id, schema)
        .context("Failed to create database table for solution set.")?;
    db.select_table(&id)
        .context("Failed to select solution set database table.")?;

    Ok(db)
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert!(false);
    }
}
