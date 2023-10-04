//! # Cyclic Solver Module
//!
//! This module implements an cyclic solver for all applicable types of games
//! through a blanket implementation of the `CyclicallySolvable` trait.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use crate::games::CyclicallySolvable;
use crate::Value;

/* SOLVER NAME */

/// Defines this solver's name for GamesmanNova's interfaces.
const SOLVER_NAME: &str = "cyclic";

/* COMFORTER IMPLEMENTATION */

/// Indicates that a game could theoretically be solved cyclically.
pub trait CyclicSolver
{
    /// Returns the value of an arbitrary state of the game.
    fn solve(game: &Self, read: bool, write: bool) -> Value;
    /// Returns the name of this solver type.
    fn name() -> String;
}

/// Blanket implementation of the cyclic solver for all cyclically solvable
/// games.
impl<G: CyclicallySolvable> CyclicSolver for G
{
    fn solve(game: &Self, read: bool, write: bool) -> Value
    {
        todo!()
    }

    fn name() -> String
    {
        SOLVER_NAME.to_owned()
    }
}
