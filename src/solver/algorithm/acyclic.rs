//! # Strong Acyclic Solving Module
//!
//! This module implements strong acyclic solving routines.

use anyhow::{Context, Result};

use crate::solver::{Game, IUtility, IntegerUtility, Persistent, Remoteness};
use crate::target::{Player, PlayerCount};
use crate::target::Implicit;

/* DEFINITIONS */

/// TODO
pub struct Solution<const N: PlayerCount> {
    remoteness: Remoteness,
    utility: [IUtility; N], 
    player: Player,
} 

/* SOLVERS */

/// TODO
pub fn solver<const N: PlayerCount, const B: usize, G>(game: &G) -> Result<()>
where
    G: Implicit<B>
        + Game<N, B>
        + IntegerUtility<N, B>
        + Persistent<Solution<N>, B>,
{
    let mut stack = Vec::new();
    stack.push(game.source());
    while let Some(curr) = stack.pop() {
        let children = game.adjacent(curr);
        if game.retrieve(&curr)?.is_none() {
            game.persist(&curr, &Solution::default())?;
            if game.sink(curr) {
                let solution = Solution {
                    remoteness: 0,
                    utility: game.utility(curr),
                    player: game.turn(curr),
                };

                game.persist(&curr, &solution)
                    .context("Failed to persist solution of terminal state.")?;
            } else {
                stack.push(curr);
                stack.extend(
                    children
                        .iter()
                        .filter(|x| {
                            game
                                .retrieve(x)
                                .expect("Database retireval error.")
                                .is_none()
                        })
                );
            }
        } else {
            let mut optimal = Solution::default();
            let mut max_val = IUtility::MIN;
            let mut min_rem = Remoteness::MAX;
            for state in children {
                let solution = game.retrieve(&state)?
                    .expect("Algorithmic guarantee breached.");

                let rem = solution.remoteness;
                let val = solution.utility[game.turn(curr)];
                if val > max_val || (val == max_val && rem < min_rem) {
                    max_val = val;
                    min_rem = rem;
                    optimal = solution;
                }
            }

            optimal.remoteness += 1;
            game.persist(&curr, &optimal)
                .context("Failed to persist solution of medial state")?;
        }
    }
    Ok(())
}

/* UTILITY IMPLEMENTATIONS */

impl<const N: PlayerCount> Default for Solution<N> {
    fn default() -> Self {
        Self { 
            remoteness: Default::default(), 
            utility: [Default::default(); N], 
            player: Default::default(), 
        }
    }
}

#[cfg(test)]
mod test {

    use anyhow::Result;
    use super::*;

    fn acyclic_solver_on_game_1() -> Result<()> {
        todo!()
    }
}
