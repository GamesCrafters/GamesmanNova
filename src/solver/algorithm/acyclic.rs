//! # Strong Acyclic Solving Module
//!
//! This module implements strong acyclic solving routines.

use anyhow::Context;
use anyhow::Result;
use rusqlite::Statement;

use crate::game;
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
    let mut conn = game::util::database()
        .context("Failed to obtain connection to game database.")?;

    let mut tx = conn
        .transaction()
        .context("Failed to start transaction.")?;

    {
        let queries = game
            .prepare(&mut tx, mode)
            .context("Failed to prepare persistent solution.")?;

        let mut insert_stmt = tx.prepare(&queries.insert)?;
        let mut select_stmt = tx.prepare(&queries.select)?;

        backward_induction(&mut insert_stmt, &mut select_stmt, game)
            .context("Backward induction algorithm failed during execution.")?;
    }

    match mode {
        IOMode::Constructive | IOMode::Overwrite => {
            tx.commit()
                .context("Failed to commit transaction.")?;
        },
        IOMode::Forgetful => (),
    }

    Ok(())
}

fn backward_induction<const N: PlayerCount, const B: usize, G>(
    insert_stmt: &mut Statement,
    select_stmt: &mut Statement,
    game: &mut G,
) -> Result<()>
where
    G: Implicit<B> + Game<N, B> + IntegerUtility<N, B> + Persistent<N, B>,
{
    let mut stack = Vec::new();
    stack.push(game.source());
    while let Some(curr) = stack.pop() {
        let children = game.adjacent(curr);
        if game
            .select(select_stmt, &curr)?
            .is_none()
        {
            game.insert(insert_stmt, &curr, &Solution::default())?;

            if game.sink(curr) {
                let solution = Solution {
                    remoteness: 0,
                    utility: game.utility(curr),
                    player: game.turn(curr),
                };

                game.insert(insert_stmt, &curr, &solution)
                    .context("Failed to persist solution of terminal state.")?;
            } else {
                stack.push(curr);
                for x in children.iter() {
                    if game
                        .select(select_stmt, x)?
                        .is_none()
                    {
                        stack.push(*x);
                    }
                }
            }
        } else {
            let mut next = Solution::default();
            let mut max_val = IUtility::MIN;
            let mut min_rem = Remoteness::MAX;
            for state in children {
                let solved = game
                    .select(select_stmt, &state)?
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

            game.insert(insert_stmt, &curr, &solution)
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

    use crate::game::mock::Node;
    use crate::game::mock::SessionBuilder;
    use crate::node;
    use crate::test;

    use super::*;

    /// Used for storing generated visualizations of the mock games being used
    /// for testing purposes in this module under their own subdirectory.
    const MODULE_NAME: &str = "acyclic-solver-tests";

    fn test_solve<const N: PlayerCount, const B: usize, G>(
        game: &mut G,
    ) -> Result<()>
    where
        G: Implicit<B> + Game<N, B> + IntegerUtility<N, B> + Persistent<N, B>,
    {
        let mut conn = test::database()
            .context("Failed to obtain connection to test database.")?;

        let mut tx = conn
            .transaction()
            .context("Failed to start transaction.")?;

        {
            let queries = game
                .prepare(&mut tx, IOMode::Overwrite)
                .context("Failed to prepare persistent solution.")?;

            let mut insert = tx.prepare(&queries.insert)?;
            let mut select = tx.prepare(&queries.select)?;

            backward_induction(&mut insert, &mut select, game).context(
                "Backward induction algorithm failed during execution.",
            )?;
        }

        tx.commit()
            .context("Failed to commit transaction.")?;

        Ok(())
    }

    #[test]
    fn acyclic_solver_on_sample1() -> Result<()> {
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

        test_solve::<3, 8, _>(&mut g)?;
        g.visualize(MODULE_NAME)?;
        Ok(())
    }

    #[test]
    fn acyclic_solver_on_sample2() -> Result<()> {
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

        test_solve::<3, 8, _>(&mut g)?;
        g.visualize(MODULE_NAME)?;
        Ok(())
    }
}
