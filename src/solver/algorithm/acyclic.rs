//! # Strong Acyclic Solving Module
//!
//! This module implements strong acyclic solving routines.

use anyhow::{Context, Result};

use crate::solver::{Game, IUtility, IntegerUtility, Persistent, Remoteness};
use crate::game::{Player, PlayerCount};
use crate::game::Implicit;

/* DEFINITIONS */

/// TODO
pub struct Solution<const N: PlayerCount> {
    remoteness: Remoteness,
    utility: [IUtility; N], 
    player: Player,
} 

/* SOLVERS */

/// TODO
pub fn solve<const N: PlayerCount, const B: usize, G>(game: &G) -> Result<()>
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

    use crate::node;
    use crate::game::mock::SessionBuilder;
    use crate::game::mock::Node;
    use crate::game::mock;

    use super::*;

    /// Used for storing generated visualizations of the mock games being used
    /// for testing purposes in this module under their own subdirectory.
    const MODULE_NAME: &str = "acyclic-solver-tests";

    #[test]
    fn acyclic_solver_on_game_1() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(2);

        let t1 = node![1; 1, 2, 3];
        let t2 = node![2; 3, 2, 1];

        let g = SessionBuilder::new("sample1")
            .edge(&s1, &s2)?
            .edge(&s1, &s3)?
            .edge(&s2, &t1)?
            .edge(&s3, &t2)?
            .source(&s1)?
            .build()?;

        // TODO: Fix
        solve::<3, 8, mock::Session<'_>>(&g)?;
        g.visualize(MODULE_NAME)?;

        Ok(())
    }
}
