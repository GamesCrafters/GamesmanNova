//! # Tier Solver Module
//!
//! This module implements a tier solver for all applicable types of games
//! through a blanket implementation of the `TierSolvable` trait.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use super::TierSolvable;
use crate::Value;

/* SOLVER NAME */

/// Defines this solver's name for GamesmanNova's interfaces.
static SOLVER_NAME: &str = "tier";

/* COMFORTER IMPLEMENTATION */

/// Indicates that a game has the capacity to perform a tier solve on itself.
pub trait TierSolve {
    /// Returns the value of an arbitrary state of the game.
    fn tier_solve(&self) -> Value;
    /// Returns the name of this solver type.
    fn tier_solver_name(&self) -> &'static str;
}

/// Blanket implementation of the tier solver for all tier solvable games.
impl<G: TierSolvable> TierSolve for G {
    fn tier_solve(&self) -> Value {
        // TODO
        Value::Win(0)
    }

    fn tier_solver_name(&self) -> &'static str {
        SOLVER_NAME
    }
}
