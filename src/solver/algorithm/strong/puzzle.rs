//! # Strong Puzzle Solving Module
//!
//! This module implements routines for strongly solving puzzles.
//!
//! ### Authorship
//! - Ishir Garg (ishirgarg@berkeley.edu)

use anyhow::{Context, Result};

use crate::database::volatile;
use crate::database::{KVStore, Tabular};
use crate::game::{Bounded, ClassicPuzzle, DTransition, Extensive, Game};
use crate::interface::IOMode;
use crate::model::SimpleUtility;
use crate::model::{Remoteness, State};
use crate::solver::error::SolverError::SolverViolation;
use crate::solver::record::sur::RecordBuffer;
use crate::solver::RecordType;
use std::collections::{HashMap, HashSet, VecDeque};

pub fn dynamic_solver<const N: usize, G>(game: &G, mode: IOMode) -> Result<()>
where
    G: DTransition<State>
        + Bounded<State>
        + ClassicPuzzle
        + Extensive<1>
        + Game,
{
    let mut db = volatile_database(game)
        .context("Failed to initialize volatile database.")?;

    reverse_bfs(&mut db, game)
        .context("Failed solving algorithm execution.")?;

    Ok(())
}

fn reverse_bfs<G, D>(db: &mut D, game: &G) -> Result<()>
where
    G: DTransition<State>
        + Bounded<State>
        + ClassicPuzzle
        + Extensive<1>
        + Game,
    D: KVStore,
{
    // Get end states and create frontiers
    let mut child_counts = discover_child_counts(db, game);
    let end_states = child_counts
        .iter()
        .filter(|&x| *x.1 == 0)
        .map(|x| *x.0);

    let mut winning_queue: VecDeque<(State, Remoteness)> = VecDeque::new();
    let mut losing_queue: VecDeque<(State, Remoteness)> = VecDeque::new();
    for end_state in end_states {
        match ClassicPuzzle::utility(game, end_state) {
            SimpleUtility::WIN => winning_queue.push_back((end_state, 0)),
            SimpleUtility::LOSE => losing_queue.push_back((end_state, 0)),
            SimpleUtility::TIE => Err(SolverViolation {
                name: "PuzzleSolver".to_string(),
                hint: format!("Primitive end position cannot have utility TIE for a puzzle"),
            })?,
            SimpleUtility::DRAW => Err(SolverViolation {
                name: "PuzzleSolver".to_string(),
                hint: format!("Primitive end position cannot have utility DRAW for a puzzle"),
            })?,
        }
    }

    // Contains states that have already been visited
    let mut visited = HashSet::new();

    // Perform BFS on winning states
    while let Some((state, remoteness)) = winning_queue.pop_front() {
        let mut buf = RecordBuffer::new(1)
            .context("Failed to create placeholder record.")?;
        buf.set_utility([SimpleUtility::WIN])
            .context("Failed to set remoteness for state.")?;
        buf.set_remoteness(remoteness)
            .context("Failed to set remoteness for state.")?;
        db.put(state, &buf);

        child_counts.insert(state, 0);
        visited.insert(state);
        let parents = game.retrograde(state);

        for parent in parents {
            if !visited.contains(&parent) {
                winning_queue.push_back((parent, remoteness + 1));
            }
        }
    }

    // Perform BFS on losing states
    while let Some((state, remoteness)) = losing_queue.pop_front() {
        let mut buf = RecordBuffer::new(1)
            .context("Failed to create placeholder record.")?;
        buf.set_utility([SimpleUtility::LOSE])
            .context("Failed to set remoteness for state.")?;
        buf.set_remoteness(remoteness)
            .context("Failed to set remoteness for state.")?;
        db.put(state, &buf);

        visited.insert(state);
        let parents = game.retrograde(state);

        for parent in parents {
            // The check below is needed, because it is theoretically possible
            // for child_counts to NOT contain a position discovered by
            // retrograde(). Consider a 3-node game tree with starting vertex 1,
            // and edges (1 -> 2), (3 -> 2), where 2 is a losing primitive
            // ending position. In this case, running discover_child_counts() on
            // 1 above only gets child_counts for states 1 and 2, however
            // calling retrograde on end state 2 in this BFS portion will
            // discover state 2 for the first time.
            match child_counts.get(&parent) {
                Some(count) => child_counts.insert(parent, count - 1),
                None => {
                    child_counts.insert(parent, game.prograde(parent).len() - 1)
                },
            };

            if !visited.contains(&parent)
                && *child_counts.get(&state).unwrap() == 0
            {
                losing_queue.push_back((parent, remoteness + 1));
            }
        }
    }

    // Get remaining draw positions
    for (state, count) in child_counts {
        if count > 0 {
            let mut buf = RecordBuffer::new(1)
                .context("Failed to create placeholder record.")?;
            buf.set_utility([SimpleUtility::DRAW])
                .context("Failed to set remoteness for state.")?;
            db.put(state, &buf);
        }
    }

    Ok(())
}

fn discover_child_counts<G, D>(db: &mut D, game: &G) -> HashMap<State, usize>
where
    G: DTransition<State>
        + Bounded<State>
        + ClassicPuzzle
        + Extensive<1>
        + Game,
    D: KVStore,
{
    let mut child_counts = HashMap::new();

    discover_child_counts_helper(db, game, game.start(), &mut child_counts);

    child_counts
}

fn discover_child_counts_helper<G, D>(
    db: &mut D,
    game: &G,
    state: State,
    child_counts: &mut HashMap<State, usize>,
) where
    G: DTransition<State> + Bounded<State> + ClassicPuzzle,
    D: KVStore,
{
    child_counts.insert(state, game.prograde(state).len());

    for child in game.prograde(state) {
        if !child_counts.contains_key(&child) {
            discover_child_counts_helper(db, game, child, child_counts);
        }
    }
}

/* DATABASE INITIALIZATION */

/// Initializes a volatile database, creating a table schema according to the
/// solver record layout, initializing a table with that schema, and switching
/// to that table before returning the database handle.
fn volatile_database<const N: usize, G>(game: &G) -> Result<volatile::Database>
where
    G: Extensive<N> + Game,
{
    let id = game.id();
    let db = volatile::Database::initialize();

    let schema = RecordType::SUR(1)
        .try_into()
        .context("Failed to create table schema for solver records.")?;
    db.create_table(&id, schema)
        .context("Failed to create database table for solution set.")?;
    db.select_table(&id)
        .context("Failed to select solution set database table.")?;

    Ok(db)

    // This is only for testing purposes
}

#[cfg(test)]
mod tests {
    use crate::game::{
        Bounded, ClassicPuzzle, DTransition, Extensive, Game, GameData,
        SimpleSum,
    };
    use crate::interface::{IOMode, SolutionMode};
    use crate::model::SimpleUtility;
    use crate::model::{State, Turn};
    use anyhow::Result;
    use std::collections::{HashMap, VecDeque};

    use super::{discover_child_counts, volatile_database};

    struct GameNode {
        utility: Option<SimpleUtility>, // Is None for non-primitive puzzle nodes
        children: Vec<State>,
    }

    struct PuzzleGraph {
        adj_list: Vec<GameNode>,
    }

    impl PuzzleGraph {
        fn size(&self) -> u64 {
            self.adj_list.len() as u64
        }
    }

    impl Game for PuzzleGraph {
        fn new(variant: Option<String>) -> Result<Self>
        where
            Self: Sized,
        {
            unimplemented!();
        }

        fn id(&self) -> String {
            String::from("GameGraph")
        }

        fn info(&self) -> GameData {
            unimplemented!();
        }

        fn solve(&self, mode: IOMode, method: SolutionMode) -> Result<()> {
            unimplemented!();
        }
    }

    impl Bounded<State> for PuzzleGraph {
        fn start(&self) -> u64 {
            0
        }

        fn end(&self, state: State) -> bool {
            self.adj_list[state as usize]
                .children
                .is_empty()
        }
    }

    impl Extensive<1> for PuzzleGraph {
        fn turn(&self, state: State) -> Turn {
            0
        }
    }

    impl ClassicPuzzle for PuzzleGraph {
        fn utility(&self, state: State) -> SimpleUtility {
            self.adj_list[state as usize]
                .utility
                .unwrap()
        }
    }

    impl DTransition<State> for PuzzleGraph {
        fn prograde(&self, state: State) -> Vec<State> {
            self.adj_list[state as usize]
                .children
                .clone()
        }

        fn retrograde(&self, state: State) -> Vec<State> {
            (0..self.size())
                .filter(|&s| {
                    self.adj_list[s as usize]
                        .children
                        .contains(&state)
                })
                .collect()
        }
    }

    #[test]
    fn gets_child_counts_correctly() -> Result<()> {
        let graph = PuzzleGraph {
            adj_list: vec![
                GameNode {
                    utility: None,
                    children: vec![1],
                },
                GameNode {
                    utility: Some(SimpleUtility::LOSE),
                    children: vec![],
                },
            ],
        };

        let mut db = volatile_database(&graph)?;

        let child_counts = discover_child_counts(&mut db, &graph);

        assert_eq!(child_counts, HashMap::from([(0, 1), (1, 0)]));

        Ok(())
    }
}
