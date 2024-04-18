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
use crate::solver::record::surcc::{ChildCount, RecordBuffer};
use bitvec::{order::Msb0, prelude::*, slice::BitSlice, store::BitStore};
use std::collections::{HashMap, HashSet, VecDeque};

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
    let end_states = discover_child_counts(db, game)?;

    let mut winning_queue: VecDeque<State> = VecDeque::new();
    let mut losing_queue: VecDeque<State> = VecDeque::new();
    for end_state in end_states {
        match ClassicPuzzle::utility(game, end_state) {
            SimpleUtility::WIN => winning_queue.push_back(end_state),
            SimpleUtility::LOSE => losing_queue.push_back(end_state),
            SimpleUtility::TIE => Err(SolverViolation {
                name: "PuzzleSolver".to_string(),
                hint: format!("Primitive end position cannot have utility TIE
                              for a puzzle"),
            })?,
            SimpleUtility::DRAW => Err(SolverViolation {
                name: "PuzzleSolver".to_string(),
                hint: format!("Primitive end position cannot have utility DRAW
                              for a puzzle"),
            })?,
        }
        // Add ending state utility and remoteness to database
        update_db_record(db, end_state, game.utility(end_state), 0, 0)?;
    }

    // Perform BFS on winning states
    while let Some(state) = winning_queue.pop_front() {
        let buf = RecordBuffer::from(db.get(state).unwrap())?;
        let child_remoteness =
            RecordBuffer::from(db.get(state).unwrap())?.get_remoteness();

        for parent in game.retrograde(state) {
            let child_count =
                RecordBuffer::from(db.get(parent).unwrap())?.get_child_count();
            if child_count > 0 {
                winning_queue.push_back(parent);
                update_db_record(
                    db,
                    parent,
                    SimpleUtility::WIN,
                    1 + child_remoteness,
                    0,
                )?;
            }
        }
    }

    // Perform BFS on losing states, where remoteness is the longest path to a
    // losing primitive
    // position.
    while let Some(state) = losing_queue.pop_front() {
        let parents = game.retrograde(state);
        let child_remoteness =
            RecordBuffer::from(db.get(state).unwrap())?.get_remoteness();

        for parent in parents {
            let child_count =
                RecordBuffer::from(db.get(parent).unwrap())?.get_child_count();
            if child_count > 0 {
                // Update child count
                let mut buf = RecordBuffer::from(db.get(parent).unwrap())
                    .context("Failed to get record for middle state")?;
                let new_child_count = buf.get_child_count() - 1;
                buf.set_child_count(new_child_count)?;
                db.put(parent, &buf);

                // If all children have been solved, set this state as a losing
                // state
                if new_child_count == 0 {
                    losing_queue.push_back(parent);
                    update_db_record(
                        db,
                        parent,
                        SimpleUtility::LOSE,
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
fn update_db_record<D>(
    db: &mut D,
    state: State,
    utility: SimpleUtility,
    remoteness: Remoteness,
    child_count: ChildCount,
) -> Result<()>
where
    D: KVStore,
{
    let mut buf = RecordBuffer::from(db.get(state).unwrap())
        .context("Failed to create record for middle state")?;
    buf.set_utility([utility])
        .context("Failed to set utility for state.")?;
    buf.set_remoteness(remoteness)
        .context("Failed to set remoteness for state.")?;
    buf.set_child_count(child_count)
        .context("Failed to set child count for state.")?;
    db.put(state, &buf);

    Ok(())
}

fn discover_child_counts<G, D>(db: &mut D, game: &G) -> Result<Vec<State>>
where
    G: DTransition<State>
        + Bounded<State>
        + ClassicPuzzle
        + Extensive<1>
        + Game,
    D: KVStore,
{
    let mut end_states = Vec::new();
    discover_child_counts_helper(db, game, game.start(), &mut end_states)?;

    Ok(end_states)
}

/// Adds child counts for each position to the database
/// Also returns a vector of all primitive positions
fn discover_child_counts_helper<G, D>(
    db: &mut D,
    game: &G,
    state: State,
    end_states: &mut Vec<State>,
) -> Result<()>
where
    G: DTransition<State>
        + Bounded<State>
        + ClassicPuzzle
        + Extensive<1>
        + Game,
    D: KVStore,
{
    let child_count = game.prograde(state).len() as ChildCount;

    if child_count == 0 {
        end_states.push(state);
    }

    // Initialize all utilies to draw; any utilities not set by the end must be
    // a drawn position
    let mut buf =
        RecordBuffer::new(1).context("Failed to create record for state")?;
    buf.set_utility([SimpleUtility::DRAW])
        .context("Failed to set remoteness for state")?;
    buf.set_child_count(child_count)
        .context("Failed to set child count for state.")?;
    db.put(state, &buf);

    // We need to check both prograde and retrograde; consider a game with 3
    // nodes where 0-->2 and 1-->2. Then, starting from node 0 with only
    // progrades would discover states 0 and 1; we need to include retrogrades
    // to discover state 2.
    for &child in game
        .prograde(state)
        .iter()
        .chain(game.retrograde(state).iter())
    {
        if db.get(child).is_none() {
            discover_child_counts_helper(db, game, child, end_states)?;
        }
    }

    Ok(())
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
    memory: HashMap<State, BitVec<u8, Msb0>>,
}

impl TestDB {
    fn initialize() -> Self {
        Self {
            memory: HashMap::new(),
        }
    }
}

impl KVStore for TestDB {
    fn put<R: crate::database::Record>(&mut self, key: State, record: &R) {
        let new = BitVec::from(record.raw()).clone();
        self.memory.insert(key, new);
    }

    fn get(
        &self,
        key: State,
    ) -> Option<&bitvec::prelude::BitSlice<u8, bitvec::prelude::Msb0>> {
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
    use crate::database::{KVStore, Tabular};
    use crate::game::mock;
    use crate::game::{
        Bounded, ClassicPuzzle, DTransition, Extensive, Game, GameData,
        SimpleSum,
    };
    use crate::interface::{IOMode, SolutionMode};
    use crate::model::SimpleUtility;
    use crate::model::{State, Turn};
    use crate::node;
    use crate::solver::record::surcc::RecordBuffer;
    use anyhow::Result;
    use std::collections::{HashMap, VecDeque};

    use super::{
        discover_child_counts, reverse_bfs_solver, volatile_database, TestDB,
    };

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
            adj_list: vec![GameNode {
                children: vec![],
                utility: Some(SimpleUtility::WIN),
            }],
        };

        // Solve game
        let mut db = volatile_database(&graph)?;
        reverse_bfs_solver(&mut db, &graph);

        assert!(matches!(
            RecordBuffer::from(db.get(0).unwrap())?.get_utility(0)?,
            SimpleUtility::WIN
        ));
        assert_eq!(
            RecordBuffer::from(db.get(0).unwrap())?.get_remoteness(),
            0
        );

        Ok(())
    }

    #[test]
    fn game_with_two_nodes_win() -> Result<()> {
        let graph = PuzzleGraph {
            adj_list: vec![
                GameNode {
                    children: vec![1],
                    utility: None,
                },
                GameNode {
                    children: vec![],
                    utility: Some(SimpleUtility::WIN),
                },
            ],
        };

        // Solve game
        let mut db = volatile_database(&graph)?;
        reverse_bfs_solver(&mut db, &graph);

        assert!(matches!(
            RecordBuffer::from(db.get(0).unwrap())?.get_utility(0)?,
            SimpleUtility::WIN
        ));
        assert!(matches!(
            RecordBuffer::from(db.get(1).unwrap())?.get_utility(0)?,
            SimpleUtility::WIN
        ));

        assert_eq!(
            RecordBuffer::from(db.get(0).unwrap())?.get_remoteness(),
            1
        );
        assert_eq!(
            RecordBuffer::from(db.get(1).unwrap())?.get_remoteness(),
            0
        );

        Ok(())
    }

    #[test]
    fn game_with_dag_win() -> Result<()> {
        let graph = PuzzleGraph {
            adj_list: vec![
                GameNode {
                    children: vec![1, 2, 4],
                    utility: None,
                },
                GameNode {
                    children: vec![3],
                    utility: None,
                },
                GameNode {
                    children: vec![3, 4],
                    utility: None,
                },
                GameNode {
                    children: vec![4],
                    utility: None,
                },
                GameNode {
                    children: vec![],
                    utility: Some(SimpleUtility::WIN),
                },
            ],
        };

        // Solve game
        let mut db = volatile_database(&graph)?;
        reverse_bfs_solver(&mut db, &graph);

        for i in 0..5 {
            assert!(matches!(
                RecordBuffer::from(db.get(i).unwrap())?.get_utility(0)?,
                SimpleUtility::WIN
            ));
        }

        assert_eq!(
            RecordBuffer::from(db.get(0).unwrap())?.get_remoteness(),
            1
        );
        assert_eq!(
            RecordBuffer::from(db.get(1).unwrap())?.get_remoteness(),
            2
        );
        assert_eq!(
            RecordBuffer::from(db.get(2).unwrap())?.get_remoteness(),
            1
        );
        assert_eq!(
            RecordBuffer::from(db.get(3).unwrap())?.get_remoteness(),
            1
        );
        assert_eq!(
            RecordBuffer::from(db.get(4).unwrap())?.get_remoteness(),
            0
        );

        Ok(())
    }

    #[test]
    fn game_with_cyclic_graph_draw() -> Result<()> {
        let graph = PuzzleGraph {
            adj_list: vec![
                GameNode {
                    children: vec![1, 2, 4],
                    utility: None,
                },
                GameNode {
                    children: vec![3],
                    utility: None,
                },
                GameNode {
                    children: vec![3, 4],
                    utility: None,
                },
                GameNode {
                    children: vec![4],
                    utility: None,
                },
                GameNode {
                    children: vec![5],
                    utility: None,
                },
                GameNode {
                    children: vec![2, 4],
                    utility: None,
                },
            ],
        };

        // Solve game
        let mut db = volatile_database(&graph)?;
        reverse_bfs_solver(&mut db, &graph);

        for i in 0..5 {
            assert!(matches!(
                RecordBuffer::from(db.get(i).unwrap())?.get_utility(0)?,
                SimpleUtility::DRAW
            ));
        }

        Ok(())
    }

    #[test]
    fn game_with_dag_win_and_lose() -> Result<()> {
        let graph = PuzzleGraph {
            adj_list: vec![
                GameNode {
                    utility: None,
                    children: vec![3],
                },
                GameNode {
                    utility: None,
                    children: vec![4],
                },
                GameNode {
                    utility: None,
                    children: vec![4],
                },
                GameNode {
                    utility: None,
                    children: vec![4, 5],
                },
                GameNode {
                    utility: None,
                    children: vec![8, 0],
                },
                GameNode {
                    utility: Some(SimpleUtility::WIN),
                    children: vec![],
                },
                GameNode {
                    utility: None,
                    children: vec![8],
                },
                GameNode {
                    utility: None,
                    children: vec![6, 8],
                },
                GameNode {
                    utility: Some(SimpleUtility::LOSE),
                    children: vec![],
                },
            ],
        };

        // Solve game
        let mut db = volatile_database(&graph)?;
        reverse_bfs_solver(&mut db, &graph);

        for i in 0..=5 {
            assert!(matches!(
                RecordBuffer::from(db.get(i).unwrap())?.get_utility(0)?,
                SimpleUtility::WIN
            ));
        }
        assert!(matches!(
            RecordBuffer::from(db.get(6).unwrap())?.get_utility(0)?,
            SimpleUtility::LOSE
        ));
        assert!(matches!(
            RecordBuffer::from(db.get(7).unwrap())?.get_utility(0)?,
            SimpleUtility::LOSE
        ));
        assert!(matches!(
            RecordBuffer::from(db.get(8).unwrap())?.get_utility(0)?,
            SimpleUtility::LOSE
        ));

        assert_eq!(
            RecordBuffer::from(db.get(0).unwrap())?.get_remoteness(),
            2
        );
        assert_eq!(
            RecordBuffer::from(db.get(1).unwrap())?.get_remoteness(),
            4
        );
        assert_eq!(
            RecordBuffer::from(db.get(2).unwrap())?.get_remoteness(),
            4
        );
        assert_eq!(
            RecordBuffer::from(db.get(3).unwrap())?.get_remoteness(),
            1
        );
        assert_eq!(
            RecordBuffer::from(db.get(4).unwrap())?.get_remoteness(),
            3
        );
        assert_eq!(
            RecordBuffer::from(db.get(5).unwrap())?.get_remoteness(),
            0
        );
        assert_eq!(
            RecordBuffer::from(db.get(6).unwrap())?.get_remoteness(),
            1
        );
        assert_eq!(
            RecordBuffer::from(db.get(7).unwrap())?.get_remoteness(),
            2
        );
        assert_eq!(
            RecordBuffer::from(db.get(8).unwrap())?.get_remoteness(),
            0
        );

        Ok(())
    }

    #[test]
    fn game_with_wld() -> Result<()> {
        let graph = PuzzleGraph {
            adj_list: vec![
                GameNode {
                    utility: None,
                    children: vec![3],
                },
                GameNode {
                    utility: None,
                    children: vec![4, 5],
                },
                GameNode {
                    utility: None,
                    children: vec![4],
                },
                GameNode {
                    utility: None,
                    children: vec![4, 5],
                },
                GameNode {
                    utility: None,
                    children: vec![8, 0],
                },
                GameNode {
                    utility: Some(SimpleUtility::WIN),
                    children: vec![],
                },
                GameNode {
                    utility: None,
                    children: vec![8],
                },
                GameNode {
                    utility: None,
                    children: vec![6, 8, 13],
                },
                GameNode {
                    utility: Some(SimpleUtility::LOSE),
                    children: vec![],
                },
                GameNode {
                    utility: Some(SimpleUtility::LOSE),
                    children: vec![10],
                },
                GameNode {
                    utility: Some(SimpleUtility::LOSE),
                    children: vec![11],
                },
                GameNode {
                    utility: Some(SimpleUtility::LOSE),
                    children: vec![9, 2],
                },
                GameNode {
                    utility: Some(SimpleUtility::LOSE),
                    children: vec![7],
                },
                GameNode {
                    utility: Some(SimpleUtility::LOSE),
                    children: vec![12],
                },
            ],
        };

        // Solve game
        let mut db = volatile_database(&graph)?;
        reverse_bfs_solver(&mut db, &graph);

        for i in 0..=5 {
            assert!(matches!(
                RecordBuffer::from(db.get(i).unwrap())?.get_utility(0)?,
                SimpleUtility::WIN
            ));
        }
        assert!(matches!(
            RecordBuffer::from(db.get(6).unwrap())?.get_utility(0)?,
            SimpleUtility::LOSE
        ));
        assert!(matches!(
            RecordBuffer::from(db.get(7).unwrap())?.get_utility(0)?,
            SimpleUtility::DRAW
        ));
        assert!(matches!(
            RecordBuffer::from(db.get(8).unwrap())?.get_utility(0)?,
            SimpleUtility::LOSE
        ));
        for i in 9..=11 {
            assert!(matches!(
                RecordBuffer::from(db.get(i).unwrap())?.get_utility(0)?,
                SimpleUtility::WIN
            ));
        }
        assert!(matches!(
            RecordBuffer::from(db.get(12).unwrap())?.get_utility(0)?,
            SimpleUtility::DRAW
        ));
        assert!(matches!(
            RecordBuffer::from(db.get(13).unwrap())?.get_utility(0)?,
            SimpleUtility::DRAW
        ));

        assert_eq!(
            RecordBuffer::from(db.get(0).unwrap())?.get_remoteness(),
            2
        );
        assert_eq!(
            RecordBuffer::from(db.get(1).unwrap())?.get_remoteness(),
            1
        );
        assert_eq!(
            RecordBuffer::from(db.get(2).unwrap())?.get_remoteness(),
            4
        );
        assert_eq!(
            RecordBuffer::from(db.get(3).unwrap())?.get_remoteness(),
            1
        );
        assert_eq!(
            RecordBuffer::from(db.get(4).unwrap())?.get_remoteness(),
            3
        );
        assert_eq!(
            RecordBuffer::from(db.get(5).unwrap())?.get_remoteness(),
            0
        );
        assert_eq!(
            RecordBuffer::from(db.get(6).unwrap())?.get_remoteness(),
            1
        );
        assert_eq!(
            RecordBuffer::from(db.get(8).unwrap())?.get_remoteness(),
            0
        );
        assert_eq!(
            RecordBuffer::from(db.get(9).unwrap())?.get_remoteness(),
            7
        );
        assert_eq!(
            RecordBuffer::from(db.get(10).unwrap())?.get_remoteness(),
            6
        );
        assert_eq!(
            RecordBuffer::from(db.get(11).unwrap())?.get_remoteness(),
            5
        );

        Ok(())
    }
}
