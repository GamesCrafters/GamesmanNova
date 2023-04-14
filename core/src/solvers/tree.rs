//! # Tree Solver Module
//!
//! This module implements a tree solver for all applicable types of games
//! through a blanket implementation of the `TreeSolvable` trait.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use super::TreeSolvable;
use crate::Value;

/* SOLVER NAME */

/// Defines this solver's name for GamesmanNova's interfaces.
static SOLVER_NAME: &str = "tree";

/* COMFORTER IMPLEMENTATION */

/// Indicates that a game has the capacity to perform a tree solve on itself.
pub trait TreeSolver {
    /// Returns the value of an arbitrary state of the game.
    fn tree_solve(game: &Self) -> Value;
    /// Returns the name of this solver type.
    fn tree_solver_name() -> String;
}

/// Blanket implementation of the tree solver for all tree solvable games.
impl<G: TreeSolvable> TreeSolver for G {
    fn tree_solve(game: &Self) -> Value {
        // TODO
        Value::Win(0)
    }

    fn tree_solver_name() -> String {
        SOLVER_NAME.to_owned()
    }
}
