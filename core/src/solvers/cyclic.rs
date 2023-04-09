//! # Cyclic Solver Module
//!
//! This module implements an cyclic solver for all applicable types of games
//! through a blanket implementation of the `CyclicallySolvable` trait.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use super::CyclicallySolvable;
use crate::Value;

/* SOLVER NAME */

/// Defines this solver's name for GamesmanNova's interfaces.
static SOLVER_NAME: &str = "cyclic";

/* COMFORTER IMPLEMENTATION */

/// Indicates that a game has the capacity to perform an cyclic solve on itself.
pub trait CyclicSolve {
    /// Returns the value of an arbitrary state of the game.
    fn cyclic_solve(&self) -> Value;
    /// Returns the name of this solver type.
    fn cyclic_solver_name() -> &'static str;
}

/// Blanket implementation of the cyclic solver for all cyclically solvable games.
impl<G: CyclicallySolvable> CyclicSolve for G {
    fn cyclic_solve(&self) -> Value {
        // TODO
        Value::Win(0)
    }

    fn cyclic_solver_name() -> &'static str {
        SOLVER_NAME
    }
}
