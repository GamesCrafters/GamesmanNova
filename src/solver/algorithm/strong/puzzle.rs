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
use std::collections::{HashMap, HashSet, VecDeque};
use bitvec::{order::Msb0, prelude::*, slice::BitSlice, store::BitStore}; 
use crate::solver::algorithm::record::surcc::RecordBuffer;

pub fn dynamic_solver<G>(game: &G, mode: IOMode) -> Result<()>
where
    G: DTransition<State>
        + Bounded<State>
        + ClassicPuzzle
        + Extensive<1>
        + Game,
{
    let mut db = volatile_database(game)
        .context("Failed to initialize volatile database.")?;

    reverse_bfs_solver(&mut db, game)
        .context("Failed solving algorithm execution.")?;

    Ok(())
}

fn reverse_bfs_solver<G, D>(db: &mut D, game: &G) -> Result<()>
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
    // Get all states with 0 child count (primitive states)
    let end_states = child_counts
        .iter()
        .filter(|&x| *x.1 == 0)
        .map(|x| *x.0);

    // Contains states that have already been visited
    let mut visited = HashSet::new();

    // TODO: Change this to no longer store remoteness, just query db
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
        visited.insert(end_state);
    }

    // Perform BFS on winning states
    while let Some((state, remoteness)) = winning_queue.pop_front() {
        let mut buf = RecordBuffer::new(1)
            .context("Failed to create placeholder record.")?;
        buf.set_utility([SimpleUtility::WIN])
            .context("Failed to set remoteness for state.")?;
        buf.set_remoteness(remoteness)
            .context("Failed to set remoteness for state.")?;
        db.put(state, &buf);

        // Zero out child counts so it doesn't get detected as draw
        child_counts.insert(state, 0);

        for parent in game.retrograde(state) {
            if !visited.contains(&parent) {
                winning_queue.push_back((parent, remoteness + 1));
                visited.insert(parent);
            }
        }
    }

    // Perform BFS on losing states, where remoteness is the longest path to a losing primitive
    // position.
    while let Some((state, remoteness)) = losing_queue.pop_front() {
        let mut buf = RecordBuffer::new(1)
            .context("Failed to create placeholder record.")?;
        buf.set_utility([SimpleUtility::LOSE])
            .context("Failed to set remoteness for state.")?;
        buf.set_remoteness(remoteness)
            .context("Failed to set remoteness for state.")?;
        db.put(state, &buf);

        let parents = game.retrograde(state);

        for parent in parents {
            if !visited.contains(&parent) {
                let new_child_count = *child_counts.get(&parent).unwrap() - 1;
                child_counts.insert(parent, new_child_count);

                if new_child_count == 0 {
                    losing_queue.push_back((parent, remoteness + 1));
                    visited.insert(parent);
                }
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

    // We need to check both prograde and retrograde; consider a game with 3 nodes where 0-->2
    // and 1-->2. Then, starting from node 0 with only progrades would discover states 0 and 1; we
    // need to include retrogrades to discover state 2.
    for &child in game.prograde(state).iter().chain(game.retrograde(state).iter()) {
        if !child_counts.contains_key(&child) {
            discover_child_counts_helper(db, game, child, child_counts);
        }
    }
}

/* DATABASE INITIALIZATION */

/// Initializes a volatile database, creating a table schema according to the
/// solver record layout, initializing a table with that schema, and switching
/// to that table before returning the database handle.

/*
fn volatile_database<const N: usize, G>(game: &G) -> Result<volatile::Database>
where
    G: Extensive<N> + Game,
{
    let id = game.id();
    let db = volatile::Database::initialize();
    let db = TestDB::initialize();

    let schema = RecordType::SUR(1)
        .try_into()
        .context("Failed to create table schema for solver records.")?;
    db.create_table(&id, schema)
        .context("Failed to create database table for solution set.")?;
    db.select_table(&id)
        .context("Failed to select solution set database table.")?;

    Ok(db)

}
*/

// THIS IS ONLY FOR TESTING PURPOSES
struct TestDB {
    memory: HashMap<State, BitVec<u8, Msb0>>
}

impl TestDB {
    fn initialize() -> Self {
        Self {
            memory: HashMap::new()
        }
    }
}

impl KVStore for TestDB {
    fn put<R: crate::database::Record>(&mut self, key: State, record: &R) {
        let new = BitVec::from(record.raw()).clone();
        self.memory.insert(key, new);
    } 

    fn get(&self, key: State) -> Option<&bitvec::prelude::BitSlice<u8, bitvec::prelude::Msb0>> {
        let vec_opt = self.memory.get(&key);
        match vec_opt {
            None => None,
            Some(vect) => Some(&vect[..]),
        }
    }

    fn del(&mut self, key: State) {
        unimplemented![]; 
    }
}

fn volatile_database<const N: usize, G>(game: &G) -> Result<TestDB>
where
    G: Extensive<N> + Game,
{
    let db = TestDB::initialize();
    Ok(db)
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
    use crate::solver::record::sur::RecordBuffer;
    use crate::database::{KVStore, Tabular};
    use crate::game::mock;
    use crate::node;

    use super::{discover_child_counts, volatile_database, reverse_bfs_solver, TestDB};

    struct GameNode {
        children: Vec<State>,
        utility: Option<SimpleUtility>, // Is None for non-primitive puzzle nodes
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
    fn game_with_single_node_win() -> Result<()> {
        let graph = PuzzleGraph {
            adj_list: vec![
                GameNode { children: vec![], utility: Some(SimpleUtility::WIN) }
            ],
        };

        // Check child counts
        let mut db = volatile_database(&graph)?;
        let child_counts = discover_child_counts(&mut db, &graph);

        assert_eq!(child_counts, HashMap::from([(0, 0)]));
        
        // Solve game
        let mut db = volatile_database(&graph)?;
        reverse_bfs_solver(&mut db, &graph);

        matches!(RecordBuffer::from(db.get(0).unwrap())?.get_utility(0)?, SimpleUtility::WIN);
        assert_eq!(RecordBuffer::from(db.get(0).unwrap())?.get_remoteness(), 0);
        
        Ok(())
    }

    #[test]
    fn game_with_two_nodes_win() -> Result<()> {
        let graph = PuzzleGraph {
            adj_list: vec![
                GameNode { children: vec![1], utility: None },
                GameNode { children: vec![], utility: Some(SimpleUtility::WIN) },
            ],
        };

        // Check child counts
        let mut db = volatile_database(&graph)?;
        let child_counts = discover_child_counts(&mut db, &graph);

        assert_eq!(child_counts, HashMap::from([(0, 1), (1, 0)]));

        // Solve game
        let mut db = volatile_database(&graph)?;
        reverse_bfs_solver(&mut db, &graph);

        matches!(RecordBuffer::from(db.get(0).unwrap())?.get_utility(0)?, SimpleUtility::WIN);
        matches!(RecordBuffer::from(db.get(1).unwrap())?.get_utility(0)?, SimpleUtility::WIN);

        assert_eq!(RecordBuffer::from(db.get(0).unwrap())?.get_remoteness(), 1);
        assert_eq!(RecordBuffer::from(db.get(1).unwrap())?.get_remoteness(), 0);

        Ok(())
    }

    #[test]
    fn game_with_dag_win() -> Result<()> {
        let graph = PuzzleGraph {
            adj_list: vec![
                GameNode { children: vec![1, 2, 4], utility: None },
                GameNode { children: vec![3], utility: None },
                GameNode { children: vec![3, 4], utility: None },
                GameNode { children: vec![4], utility: None },
                GameNode { children: vec![], utility: Some(SimpleUtility::WIN) },
            ],
        };

        // Check child counts
        let mut db = volatile_database(&graph)?;
        let child_counts = discover_child_counts(&mut db, &graph);

        assert_eq!(child_counts, HashMap::from([(0, 3), (1, 1), (2, 2), (3, 1), (4, 0)]));

        // Solve game
        let mut db = volatile_database(&graph)?;
        reverse_bfs_solver(&mut db, &graph);

        for i in 0..5 {
            matches!(RecordBuffer::from(db.get(i).unwrap())?.get_utility(0)?, SimpleUtility::WIN);
        }

        assert_eq!(RecordBuffer::from(db.get(0).unwrap())?.get_remoteness(), 1);
        assert_eq!(RecordBuffer::from(db.get(1).unwrap())?.get_remoteness(), 2);
        assert_eq!(RecordBuffer::from(db.get(2).unwrap())?.get_remoteness(), 1);
        assert_eq!(RecordBuffer::from(db.get(3).unwrap())?.get_remoteness(), 1);
        assert_eq!(RecordBuffer::from(db.get(4).unwrap())?.get_remoteness(), 0);

        Ok(())
    }

    #[test]
    fn game_with_cyclic_graph_draw() -> Result<()> {
        let graph = PuzzleGraph {
            adj_list: vec![
                GameNode { children: vec![1, 2, 4], utility: None },
                GameNode { children: vec![3], utility: None },
                GameNode { children: vec![3, 4], utility: None },
                GameNode { children: vec![4], utility: None },
                GameNode { children: vec![5], utility: None },
                GameNode { children: vec![2, 4], utility: None },
            ],
        };


        // Check child counts
        let mut db = volatile_database(&graph)?;
        let child_counts = discover_child_counts(&mut db, &graph);

        assert_eq!(child_counts, HashMap::from([(0, 3), (1, 1), (2, 2), (3, 1), (4, 1), (5, 2)]));

        // Solve game
        let mut db = volatile_database(&graph)?;
        reverse_bfs_solver(&mut db, &graph);

        for i in 0..5 {
            matches!(RecordBuffer::from(db.get(i).unwrap())?.get_utility(0)?, SimpleUtility::TIE);
        }

        Ok(())
    }

    #[test]
    fn game_with_dag_win_and_lose() -> Result<()> {
        let graph = PuzzleGraph {
            adj_list: vec![
                GameNode { utility: None, children: vec![3] },
                GameNode { utility: None, children: vec![4] },
                GameNode { utility: None, children: vec![4] },
                GameNode { utility: None, children: vec![4, 5] },
                GameNode { utility: None, children: vec![8, 0] },
                GameNode { utility: Some(SimpleUtility::WIN), children: vec![] },
                GameNode { utility: None, children: vec![8] },
                GameNode { utility: None, children: vec![6, 8] },
                GameNode { utility: Some(SimpleUtility::LOSE), children: vec![] },
            ],
        };

        // Check child counts
        let mut db = volatile_database(&graph)?;
        let child_counts = discover_child_counts(&mut db, &graph);

        assert_eq!(child_counts, HashMap::from([(0, 1), (1, 1), (2, 1), (3, 2), (4, 2), (5, 0), (6, 1), (7, 2), (8, 0)]));

        // Solve game
        let mut db = volatile_database(&graph)?;
        reverse_bfs_solver(&mut db, &graph);

        for i in 0..=5 {
            matches!(RecordBuffer::from(db.get(i).unwrap())?.get_utility(0)?, SimpleUtility::WIN);
        }
        matches!(RecordBuffer::from(db.get(6).unwrap())?.get_utility(0)?, SimpleUtility::LOSE);
        matches!(RecordBuffer::from(db.get(7).unwrap())?.get_utility(0)?, SimpleUtility::LOSE);
        matches!(RecordBuffer::from(db.get(8).unwrap())?.get_utility(0)?, SimpleUtility::LOSE);

        assert_eq!(RecordBuffer::from(db.get(0).unwrap())?.get_remoteness(), 2);
        assert_eq!(RecordBuffer::from(db.get(1).unwrap())?.get_remoteness(), 4);
        assert_eq!(RecordBuffer::from(db.get(2).unwrap())?.get_remoteness(), 4);
        assert_eq!(RecordBuffer::from(db.get(3).unwrap())?.get_remoteness(), 1);
        assert_eq!(RecordBuffer::from(db.get(4).unwrap())?.get_remoteness(), 3);
        assert_eq!(RecordBuffer::from(db.get(5).unwrap())?.get_remoteness(), 0);
        assert_eq!(RecordBuffer::from(db.get(6).unwrap())?.get_remoteness(), 1);
        assert_eq!(RecordBuffer::from(db.get(7).unwrap())?.get_remoteness(), 2);
        assert_eq!(RecordBuffer::from(db.get(8).unwrap())?.get_remoteness(), 0);
        
        Ok(())
    }

    #[test]
    fn game_with_wld() -> Result<()> {
        let graph = PuzzleGraph {
            adj_list: vec![
                GameNode { utility: None, children: vec![3] },
                GameNode { utility: None, children: vec![4, 5] },
                GameNode { utility: None, children: vec![4] },
                GameNode { utility: None, children: vec![4, 5] },
                GameNode { utility: None, children: vec![8, 0] },
                GameNode { utility: Some(SimpleUtility::WIN), children: vec![] },

                GameNode { utility: None, children: vec![8] },
                GameNode { utility: None, children: vec![6, 8, 13] },
                GameNode { utility: Some(SimpleUtility::LOSE), children: vec![] },

                GameNode { utility: Some(SimpleUtility::LOSE), children: vec![10] },
                GameNode { utility: Some(SimpleUtility::LOSE), children: vec![11] },
                GameNode { utility: Some(SimpleUtility::LOSE), children: vec![9, 2] },

                GameNode { utility: Some(SimpleUtility::LOSE), children: vec![7] },
                GameNode { utility: Some(SimpleUtility::LOSE), children: vec![12] },
            ],
        };

        // Solve game
        let mut db = volatile_database(&graph)?;
        reverse_bfs_solver(&mut db, &graph);

        for i in 0..=5 {
            matches!(RecordBuffer::from(db.get(i).unwrap())?.get_utility(0)?, SimpleUtility::WIN);
        }
        matches!(RecordBuffer::from(db.get(6).unwrap())?.get_utility(0)?, SimpleUtility::LOSE);
        matches!(RecordBuffer::from(db.get(7).unwrap())?.get_utility(0)?, SimpleUtility::DRAW);
        matches!(RecordBuffer::from(db.get(8).unwrap())?.get_utility(0)?, SimpleUtility::LOSE);
        for i in 9..=11 {
            matches!(RecordBuffer::from(db.get(i).unwrap())?.get_utility(0)?, SimpleUtility::WIN);
        }
        matches!(RecordBuffer::from(db.get(12).unwrap())?.get_utility(0)?, SimpleUtility::DRAW);
        matches!(RecordBuffer::from(db.get(13).unwrap())?.get_utility(0)?, SimpleUtility::DRAW);

        assert_eq!(RecordBuffer::from(db.get(0).unwrap())?.get_remoteness(), 2);
        assert_eq!(RecordBuffer::from(db.get(1).unwrap())?.get_remoteness(), 1);
        assert_eq!(RecordBuffer::from(db.get(2).unwrap())?.get_remoteness(), 4);
        assert_eq!(RecordBuffer::from(db.get(3).unwrap())?.get_remoteness(), 1);
        assert_eq!(RecordBuffer::from(db.get(4).unwrap())?.get_remoteness(), 3);
        assert_eq!(RecordBuffer::from(db.get(5).unwrap())?.get_remoteness(), 0);
        assert_eq!(RecordBuffer::from(db.get(6).unwrap())?.get_remoteness(), 1);
        assert_eq!(RecordBuffer::from(db.get(8).unwrap())?.get_remoteness(), 0);
        assert_eq!(RecordBuffer::from(db.get(9).unwrap())?.get_remoteness(), 7);
        assert_eq!(RecordBuffer::from(db.get(10).unwrap())?.get_remoteness(), 6);
        assert_eq!(RecordBuffer::from(db.get(11).unwrap())?.get_remoteness(), 5);
        
        Ok(())
    }
}
