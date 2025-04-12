//! # Strong Acyclic Solving Module
//!
//! This module implements strong acyclic solving routines.

use anyhow::Result;

use crate::interface::IOMode;
use crate::solver::{IntegerUtility, Sequential};
use crate::target::{Information, PlayerCount};
use crate::target::{Bounded, Transition};

/* SOLVERS */

pub fn solver<const N: PlayerCount, const B: usize, G>(
    game: &G,
    mode: IOMode,
) -> Result<()>
where
    G: Transition<B>
        + Bounded<B>
        + IntegerUtility<N, B>
        + Sequential<N, B>
        + Information,
{
    todo!()
}

/* SOLVING ALGORITHMS */

///// Performs an iterative depth-first traversal of the game tree, assigning to
///// each game `state` a remoteness and utility values for each player within
///// `table`. This uses heap-allocated memory for keeping a stack of positions to
///// facilitate DFS, as well as for communicating state transitions.
//fn backward_induction<const N: PlayerCount, const B: usize, M, G>(
//    solution: &mut M,
//    game: &G,
//) -> Result<()>
//where
//    M: ByteMap,
//    G: Transition<B> + Bounded<B> + IntegerUtility<N, B> + Sequential<N, B>,
//{
//    let mut stack = Vec::new();
//    stack.push(game.start());
//    while let Some(curr) = stack.pop() {
//        let children = game.prograde(curr);
//        let mut buf = new_record::<N>()
//            .context("Failed to create placeholder record.")?;
//
//        if solution.get(&curr)?.is_none() {
//            solution.insert(&curr, &buf)?;
//            if game.end(curr) {
//                buf = new_record::<N>()
//                    .context("Failed to create record for end state.")?;
//
//                buf.set_integer_utility(game.utility(curr))
//                    .context("Failed to copy utility values to record.")?;
//
//                buf.set_remoteness(0)
//                    .context("Failed to set remoteness for end state.")?;
//
//                solution.insert(&curr, &buf)?;
//            } else {
//                stack.push(curr);
//                stack.extend(children.iter().filter(|x| {
//                    solution
//                        .get(x)
//                        .expect("Database GET error.")
//                        .is_none()
//                }));
//            }
//        } else {
//            let mut optimal = buf;
//            let mut max_val = IUtility::MIN;
//            let mut min_rem = Remoteness::MAX;
//            for state in children {
//                let buf = new_record::<N>()
//                    .context("Failed to create record for middle state.")?;
//
//                let val = buf
//                    .get_integer_utility(game.turn(state))
//                    .context("Failed to get utility from record.")?;
//
//                let rem = buf.get_remoteness()?;
//                if val > max_val || (val == max_val && rem < min_rem) {
//                    max_val = val;
//                    min_rem = rem;
//                    optimal = buf;
//                }
//            }
//
//            optimal
//                .set_remoteness(min_rem + 1)
//                .context("Failed to set remoteness for solved record.")?;
//
//            solution.insert(&curr, &optimal)?;
//        }
//    }
//    Ok(())
//}
//
///* HELPERS */
//
///// Initialize a new record buffer with integer utility for `N` players, storing
///// additional remoteness information.
//fn new_record<const N: usize>() -> Result<mur::RecordBuffer> {
//    mur::RecordBuffer::new(N, UtilityType::Integer, true, false)
//}

#[cfg(test)]
mod test {
    // TODO
}
