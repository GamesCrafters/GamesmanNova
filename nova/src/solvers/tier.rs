//! # Tier Solver Module
//!
//! This module implements a tier solver for all applicable types of games
//! through a blanket implementation of the `TierSolvable` trait.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use crate::games::TierSolvable;
use crate::models::Value;

/* SOLVER NAME */

/// Defines this solver's name for GamesmanNova's interfaces.
const SOLVER_NAME: &str = "tier";

/* COMFORTER IMPLEMENTATION */

/// Indicates that a game could theoretically be solved by tiers.
pub trait TierSolver
{
    /// Returns the value of an arbitrary state of the game, and uses `read`
    /// and `write` for specifying I/O preferences to database implementations.
    fn solve(game: &Self, read: bool, write: bool) -> Value;
    /// Returns the name of this solver type.
    fn name() -> String;
}

/// Blanket implementation of the tier solver for all tier solvable games.
impl<G: TierSolvable> TierSolver for G
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
