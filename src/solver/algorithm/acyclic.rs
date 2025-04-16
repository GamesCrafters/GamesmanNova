//! # Strong Acyclic Solving Module
//!
//! This module implements strong acyclic solving routines.

use anyhow::Context;
use anyhow::Result;

use crate::game::Implicit;
use crate::game::PlayerCount;
use crate::interface::IOMode;
use crate::solver::Game;
use crate::solver::IUtility;
use crate::solver::IntegerUtility;
use crate::solver::Persistent;
use crate::solver::Remoteness;
use crate::solver::Solution;

/* SOLVERS */

/// Compute the game-theoretic solution to a sequential `game` through backward
/// induction over its states. Store solution according to `mode`.
pub fn solve<const N: PlayerCount, const B: usize, G>(
    game: &mut G,
    mode: IOMode,
) -> Result<()>
where
    G: Implicit<B> + Game<N, B> + IntegerUtility<N, B> + Persistent<N, B>,
{
    game.prepare(mode)
        .context("Failed to prepare persistent solution.")?;

    backward_induction(game)
        .context("Backward induction algorithm failed during execution.")?;

    match mode {
        IOMode::Constructive | IOMode::Overwrite => {
            game.commit()
                .context("Failed to commit transaction.")?;
        },
        IOMode::Forgetful => (),
    }

    Ok(())
}

fn backward_induction<const N: PlayerCount, const B: usize, G>(
    game: &mut G,
) -> Result<()>
where
    G: Implicit<B> + Game<N, B> + IntegerUtility<N, B> + Persistent<N, B>,
{
    let mut stack = Vec::new();
    stack.push(game.source());
    while let Some(curr) = stack.pop() {
        let children = game.adjacent(curr);
        if game.select(&curr)?.is_none() {
            game.insert(&curr, &Solution::default())?;
            if game.sink(curr) {
                let solution = Solution {
                    remoteness: 0,
                    utility: game.utility(curr),
                    player: game.turn(curr),
                };

                game.insert(&curr, &solution)
                    .context("Failed to persist solution of terminal state.")?;
            } else {
                stack.push(curr);
                stack.extend(children.iter().filter(|x| {
                    game.select(x)
                        .expect("Database retireval error.")
                        .is_none()
                }));
            }
        } else {
            let mut next = Solution::default();
            let mut max_val = IUtility::MIN;
            let mut min_rem = Remoteness::MAX;
            for state in children {
                let solved = game
                    .select(&state)?
                    .expect("Algorithmic guarantee breached.");

                let rem = solved.remoteness;
                let val = solved.utility[game.turn(curr)];
                if val > max_val || (val == max_val && rem < min_rem) {
                    max_val = val;
                    min_rem = rem;
                    next = solved;
                }
            }

            let solution = Solution {
                remoteness: next.remoteness + 1,
                utility: next.utility,
                player: game.turn(curr),
            };

            game.insert(&curr, &solution)
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
    use serial_test::serial;

    use crate::game::mock;
    use crate::game::mock::Node;
    use crate::game::mock::SessionBuilder;
    use crate::node;
    use crate::test;

    use super::*;

    /// Used for storing generated visualizations of the mock games being used
    /// for testing purposes in this module under their own subdirectory.
    const MODULE_NAME: &str = "acyclic-solver-tests";

    #[serial]
    #[tokio::test(flavor = "multi_thread")]
    async fn acyclic_solver_on_sample1() -> Result<()> {
        test::prepare().await?;

        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(2);

        let t1 = node![1; 1, 2, 3];
        let t2 = node![2; 3, 2, 1];

        let mut g = SessionBuilder::new("sample1")
            .edge(&s1, &s2)?
            .edge(&s1, &s3)?
            .edge(&s2, &t1)?
            .edge(&s3, &t2)?
            .source(&s1)?
            .build()?;

        solve::<3, 8, mock::Session<'_>>(&mut g, IOMode::Overwrite)?;
        g.visualize(MODULE_NAME)?;
        Ok(())
    }

    #[serial]
    #[tokio::test(flavor = "multi_thread")]
    async fn acyclic_solver_on_sample2() -> Result<()> {
        test::prepare().await?;

        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(2);
        let s4 = node!(0);
        let s5 = node!(1);
        let s6 = node!(2);

        let t1 = node![1; 1, 2, 3];
        let t2 = node![0; 3, 2, 1];

        let mut g = SessionBuilder::new("sample2")
            .edge(&s1, &s2)?
            .edge(&s2, &s3)?
            .edge(&s3, &s4)?
            .edge(&s1, &s3)?
            .edge(&s2, &s4)?
            .edge(&s2, &s5)?
            .edge(&s5, &s6)?
            .edge(&s3, &s5)?
            .edge(&s5, &t1)?
            .edge(&s6, &t2)?
            .edge(&s4, &t1)?
            .source(&s1)?
            .build()?;

        solve::<3, 8, mock::Session<'_>>(&mut g, IOMode::Overwrite)?;
        g.visualize(MODULE_NAME)?;
        Ok(())
    }
}
