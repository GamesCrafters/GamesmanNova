//! Algorithmic Module
//!
//! TODO

use std::collections::HashMap;
use std::collections::HashSet;

use anyhow::Result;

use crate::game::Implicit;
use crate::game::PlayerCount;
use crate::game::State;
use crate::game::Transpose;
use crate::solver::Component;
use crate::solver::Degree;
use crate::solver::IntegerUtility;
use crate::solver::Partition;
use crate::solver::Sequential;
use crate::solver::UtilityType;
use crate::solver::backend::Frontier;
use crate::solver::backend::record::RecordBuffer;
use crate::solver::backend::record::RecordMode;
use crate::solver::backend::scheduler::Task;

/* ALGORITHMS */

/// Depth-first search.
pub fn dfs<G, const B: usize>(
    front: Frontier<B>,
    comp: Component,
    tree: &mut sled::Tree,
    game: &G,
) -> Result<Vec<Task<B>>>
where
    G: Implicit<B> + Partition<B>,
{
    let mut stack = Vec::new();
    let mut tasks = HashMap::new();

    stack.extend(front.iter());
    while let Some(state) = stack.pop() {
        if !tree.contains_key(state)? {
            let adjacent = game.adjacent(&state);

            let degree = adjacent.len() as Degree;
            let mut record = RecordBuffer::new(RecordMode::Discovery)?;
            record.set_degree(degree)?;
            tree.insert(state, record.as_ref())?;

            for &neighbor in adjacent.iter() {
                let ncomp = game.component(&neighbor);
                if ncomp != comp {
                    let task = tasks
                        .entry(ncomp)
                        .or_insert(Task::Discover(ncomp, vec![]));

                    task.push(neighbor);
                } else {
                    stack.push(neighbor);
                }
            }
        }
    }

    let mut tasks: Vec<Task<B>> = tasks.into_values().collect();
    if tasks.is_empty() {
        tasks.push(Task::Process(comp, front))
    }

    Ok(tasks)
}

/// Value iteration.
pub fn viter<G, const N: PlayerCount, const B: usize>(
    front: Frontier<B>,
    comp: Component,
    tree: &mut sled::Tree,
    game: &G,
) -> Result<Vec<Task<B>>>
where
    G: IntegerUtility<N, B> + Sequential<N, B> + Transpose<B> + Partition<B>,
{
    let mut frontier = front;
    let mut tasks = HashMap::new();
    while let Some(state) = frontier.pop() {
        let current = get_record_unchecked::<N, B>(tree, &state)?;
        for prev in game
            .adjacent(&state)
            .iter()
            .filter(|s| tree.get(s).unwrap().is_some())
        {
            let record = get_record_unchecked::<N, B>(tree, prev)?;
            if record.is_solution() {
                let turn = game.turn(prev);
                let util = record.get_integer_utility(turn)?;
                let ante = current.get_integer_utility(turn)?;
            }

            let ncomp = game.component(prev);
            if ncomp != comp {
                let task = tasks
                    .entry(ncomp)
                    .or_insert(Task::Process(ncomp, vec![]));

                task.push(*prev);
            }
        }
    }

    let tasks = tasks.into_values().collect();
    Ok(tasks)
}

/* HELPERS */

fn get_record_unchecked<const N: usize, const B: usize>(
    tree: &sled::Tree,
    state: &State<B>,
) -> Result<RecordBuffer> {
    Ok(get_record::<N, B>(tree, state)?
        .expect("Unchecked access to missing record."))
}

fn get_record<const N: usize, const B: usize>(
    tree: &sled::Tree,
    state: &State<B>,
) -> Result<Option<RecordBuffer>> {
    if let Some(bytes) = tree.get(state)? {
        let discovery = RecordBuffer::get_discriminant(&bytes);
        if discovery {
            Ok(Some(RecordBuffer::from(
                &bytes,
                RecordMode::Discovery,
            )?))
        } else {
            Ok(Some(RecordBuffer::from(
                &bytes,
                RecordMode::Solution {
                    players: N,
                    utility: UtilityType::Integer,
                    remoteness: true,
                    draw: true,
                },
            )?))
        }
    } else {
        Ok(None)
    }
}
