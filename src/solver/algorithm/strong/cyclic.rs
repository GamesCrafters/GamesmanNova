//! # Strong Cyclic Solving Module
//!
//! This module implements strong cyclic solvers for all applicable types of
//! games through blanket implementations of the `acyclic::Solver` trait,
//! optimizing for specific game characteristics wherever possible.
//!
//! #### Authorship
//! - Max Fierro, 12/3/2023 (maxfierro@berkeley.edu)
//! - Ishir Garg, 3/12/2024 (ishirgarg@berkeley.edu)

use anyhow::{Context, Result};

use std::collections::{HashMap, VecDeque};

use crate::database::volatile;
use crate::database::{KVStore, Tabular};
use crate::game::{Bounded, DTransition, Extensive, Game, SimpleSum};
use crate::interface::IOMode;
use crate::model::SimpleUtility;
use crate::model::{PlayerCount, Remoteness, State, Turn};
use crate::solver::record::sur::RecordBuffer;
use crate::solver::RecordType;

pub fn dynamic_solver<G>(
    game: &G,
    mode: IOMode,
) -> Result<()>
where
    G: DTransition<State> + Bounded<State> + SimpleSum<2> + Extensive<2> + Game,
{
    let mut db =
        volatile_database(game).context("Failed to initialize database.")?;
    cyclic_solver(game, &mut db)?;
    Ok(())
}

fn cyclic_solver<G, D>(game: &G, db: &mut D) -> Result<()>
where
    G: DTransition<State> + Bounded<State> + SimpleSum<2> + Extensive<2> + Game,
    D: KVStore,
{
    let mut winning_frontier = VecDeque::new();
    let mut tying_frontier = VecDeque::new();
    let mut losing_frontier = VecDeque::new();

    let mut child_counts = HashMap::new();

    enqueue_children(
        &mut winning_frontier,
        &mut tying_frontier,
        &mut losing_frontier,
        game.start(),
        game,
        &mut child_counts,
        db,
    )?;

    // Process winning and losing frontiers
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

        let db_entry = RecordBuffer::from(db.get(child).unwrap())
            .context("Failed to create record for middle state.")?;
        let child_utility = db_entry
            .get_utility(game.turn(child))
            .context("Failed to get utility from record.")?;
        let child_remoteness = db_entry.get_remoteness();

        let parents = game.retrograde(child);
        // If child is a losing position
        if let SimpleUtility::LOSE = child_utility {
            for parent in parents {
                if *child_counts.get(&parent).expect("Failed to enqueue parent state in initial enqueueing stage") > 0 {
                    // Add database entry
                    let mut buf = RecordBuffer::new(game.players())
                        .context("Failed to create record for end state.")?;
                    buf.set_utility([SimpleUtility::WIN, SimpleUtility::LOSE])?;
                    buf.set_remoteness(child_remoteness + 1)?;
                    db.put(parent, &buf);

                    // Update child counts
                    child_counts.insert(parent, 0);

                    // Add parent to win frontier
                    winning_frontier.push_back(parent);
                }
            }
        }
        // If child is a winning position
        else if matches!(child_utility, SimpleUtility::WIN) {
            for parent in parents {
                let child_count = *child_counts.get(&parent).expect(
                    "Failed to enqueue parent state in initial enqueuing stage",
                );
                // Parent has already been solved
                if child_count == 0 {
                    continue;
                }
                // This is the last child left to process
                if child_count == 1 {
                    // Add database entry
                    let mut buf = RecordBuffer::new(game.players())
                        .context("Failed to create record for end state.")?;
                    buf.set_utility([SimpleUtility::LOSE, SimpleUtility::WIN])?;
                    buf.set_remoteness(child_remoteness + 1)?;
                    db.put(parent, &buf);

                    // Add parent to win frontier
                    losing_frontier.push_back(parent);
                }
                // Update child count
                child_counts.insert(parent, child_count - 1);
            }
        }
        // Child should never be a tying position
        else if matches!(child_utility, SimpleUtility::TIE) {
            for parent in parents {
                let child_count = *child_counts.get(&parent).expect(
                    "Failed to enqueue parent state in initial enqueuing stage",
                );
                // Parent has already been solved
                if child_count == 0 {
                    continue;
                }
                // Add database entry
                let mut buf = RecordBuffer::new(game.players())
                    .context("Failed to create record for end state.")?;
                buf.set_utility([SimpleUtility::TIE, SimpleUtility::TIE])?;
                buf.set_remoteness(child_remoteness + 1)?;
                db.put(parent, &buf);

                // Add parent to win frontier
                tying_frontier.push_back(parent);
                // Update child count
                child_counts.insert(parent, 0);
            }
        } else {
            panic!["Position with invalid utility found in frontiers"];
        }
    }

    // Assign drawing utility
    for (parent, child_count) in child_counts {
        if child_count > 0 {
            let mut buf = RecordBuffer::new(game.players())
                .context("Failed to create record for end state.")?;
            buf.set_utility([SimpleUtility::DRAW, SimpleUtility::DRAW])?;
            db.put(parent, &buf);
        }
    }

    Ok(())
}

/// Set up the initial frontiers and primitive position database entries
fn enqueue_children<G, D>(
    winning_frontier: &mut VecDeque<State>,
    tying_frontier: &mut VecDeque<State>,
    losing_frontier: &mut VecDeque<State>,
    curr_state: State,
    game: &G,
    child_counts: &mut HashMap<State, usize>,
    db: &mut D,
) -> Result<()>
where
    G: DTransition<State> + Bounded<State> + SimpleSum<2> + Extensive<2> + Game,
    D: KVStore,
{
    if game.end(curr_state) {
        let mut buf = RecordBuffer::new(game.players())
            .context("Failed to create placeholder record.")?;
        buf.set_utility(game.utility(curr_state))
            .context("Failed to copy utility values to record.")?;
        buf.set_remoteness(0)
            .context("Failed to set remoteness for end state.")?;
        db.put(curr_state, &buf);

        match game
            .utility(curr_state)
            .get(game.turn(curr_state))
        {
            Some(SimpleUtility::WIN) => winning_frontier.push_back(curr_state),
            Some(SimpleUtility::TIE) => tying_frontier.push_back(curr_state),
            Some(SimpleUtility::LOSE) => losing_frontier.push_back(curr_state),
            _ => {
                panic!["Utility for primitive ending position found to be draw"]
            },
        }
        return Ok(());
    }

    // Enqueue primitive positions into frontiers
    let children = game.prograde(curr_state);
    child_counts.insert(curr_state, children.len());

    for child in children {
        if child_counts.contains_key(&child) {
            continue;
        }
        enqueue_children(
            winning_frontier,
            tying_frontier,
            losing_frontier,
            child,
            game,
            child_counts,
            db,
        )?;
    }

    Ok(())
}

/* DATABASE INITIALIZATION */

/// Initializes a volatile database, creating a table schema according to the
/// solver record layout, initializing a table with that schema, and switching
/// to that table before returning the database handle.
fn volatile_database<G>(game: &G) -> Result<volatile::Database>
where
    G: Extensive<2> + Game,
{
    let id = game.id();
    let db = volatile::Database::initialize();

    let schema = RecordType::SUR(2)
        .try_into()
        .context("Failed to create table schema for solver records.")?;
    db.create_table(&id, schema)
        .context("Failed to create database table for solution set.")?;
    db.select_table(&id)
        .context("Failed to select solution set database table.")?;

    Ok(db)
}

#[cfg(test)]
mod tests {
    use crate::game::{
        Bounded, DTransition, Extensive, Game, GameData, SimpleSum,
    };
    use crate::interface::{IOMode, SolutionMode};
    use crate::model::SimpleUtility;
    use crate::model::{State, Turn};
    use anyhow::Result;
    use std::collections::{HashMap, VecDeque};

    use super::{enqueue_children, volatile_database};

    struct GameNode {
        turn: Turn,
        utility: Vec<SimpleUtility>,
        children: Vec<State>,
    }

    struct GameGraph {
        num_states: u32,
        adj_list: Vec<GameNode>,
    }

    impl Game for GameGraph {
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

    impl Bounded<State> for GameGraph {
        fn start(&self) -> u64 {
            0
        }

        fn end(&self, state: State) -> bool {
            self.adj_list[state as usize]
                .children
                .is_empty()
        }
    }

    impl Extensive<2> for GameGraph {
        fn turn(&self, state: State) -> Turn {
            self.adj_list[state as usize].turn
        }
    }

    impl SimpleSum<2> for GameGraph {
        fn utility(&self, state: State) -> [SimpleUtility; 2] {
            let util = &self.adj_list[state as usize].utility;
            [util[0], util[1]]
        }
    }

    impl DTransition<State> for GameGraph {
        fn prograde(&self, state: State) -> Vec<State> {
            self.adj_list[state as usize]
                .children
                .clone()
        }

        fn retrograde(&self, state: State) -> Vec<State> {
            todo![];
        }
    }

    #[test]
    fn enqueues_children_properly() -> Result<()> {
        let graph = GameGraph {
            num_states: 2,
            adj_list: vec![
                GameNode {
                    turn: 0,
                    utility: vec![],
                    children: vec![1],
                },
                GameNode {
                    turn: 1,
                    utility: vec![SimpleUtility::LOSE, SimpleUtility::WIN],
                    children: vec![],
                },
            ],
        };

        let mut db = volatile_database(&graph)?;

        let mut winning_frontier = VecDeque::new();
        let mut tying_frontier = VecDeque::new();
        let mut losing_frontier = VecDeque::new();
        let mut child_counts = HashMap::new();

        enqueue_children(
            &mut winning_frontier,
            &mut tying_frontier,
            &mut losing_frontier,
            graph.start(),
            &graph,
            &mut child_counts,
            &mut db,
        )?;

        assert!(winning_frontier.is_empty());
        assert!(tying_frontier.is_empty());
        assert_eq!(losing_frontier, vec![1]);
        assert_eq!(child_counts, HashMap::from([(0, 1), (1, 0)]));

        Ok(())
    }
}
