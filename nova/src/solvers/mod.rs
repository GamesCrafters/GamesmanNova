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
use std::collections::HashSet;

/* SOLVER MODULES */

pub mod acyclic;
pub mod cyclic;
pub mod tier;
pub mod tree;

/* HELPER FUNCTIONS */

/// Returns the most favorable value with the least remoteness in the case of
/// a possible win or tie, or with the greatest remoteness in the case of an
/// inevitable loss.
pub fn choose_value(available: HashSet<Value>) -> Value
{
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
