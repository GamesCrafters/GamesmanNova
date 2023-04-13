//! # Solvers Module
//!
//! `solvers` provides algorithms for solving games with state graphs that
//! have cycles, which are acyclic, which are trees, and which can be
//! partitioned into independent components called "tiers," among others.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use super::Value;
use crate::archetypes::Game;
use std::collections::HashSet;

/// Blanket implementation of a solver for all tree-like games.
pub mod tree;

/// Blanket implementation of a solver for all acyclic games.
pub mod acyclic;

/// Blanket implementation of a solver for all tiered games.
pub mod tiered;

/// Blanket implementation of a solver for all cyclic games.
pub mod cyclic;

/* SOLVING MARKER TRAITS */

/// Indicates that a game is solvable using methods only available to games
/// whose state graphs are acyclic (which includes tree games).
pub trait AcyclicallySolvable
where
    Self: Game,
{
}

/// Indicates that a game is solvable in a generally inefficient manner.
pub trait CyclicallySolvable
where
    Self: Game,
{
}

/// Indicates that a game's state graph can be partitioned into independent
/// connected components and solved taking advantage of this.
pub trait TierSolvable
where
    Self: Game,
{
}

/// Indicates that a game is solvable using methods only available to games
/// with unique move paths to all states.
pub trait TreeSolvable
where
    Self: Game,
{
}

/* HELPER FUNCTIONS */

/// Returns the most favorable value with the least remoteness in the case of
/// a possible win or tie, or with the greatest remoteness in the case of an
/// inevitable loss.
pub fn choose_value(available: HashSet<Value>) -> Value {
    let mut w_rem = u32::MAX;
    let mut t_rem = u32::MAX;
    let mut l_rem = 0;
    let mut win = false;
    let mut tie = false;
    for out in available {
        match out {
            Value::Lose(rem) => {
                win = true;
                if (rem + 1) < w_rem {
                    w_rem = rem + 1;
                }
            }
            Value::Tie(rem) => {
                tie = true;
                if (rem + 1) < t_rem {
                    t_rem = rem + 1;
                }
            }
            Value::Win(rem) => {
                if (rem + 1) > l_rem {
                    l_rem = rem + 1;
                }
            }
        }
    }
    if win {
        Value::Win(w_rem)
    } else if tie {
        Value::Tie(t_rem)
    } else {
        Value::Lose(l_rem)
    }
}
