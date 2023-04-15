#![warn(missing_docs)]

//! # GamesmanNova Games Library
//!
//! The `games` crate includes implementations for games intended to be
//! solved. To be able to solve a game, with GamesmanNova, it must satisfy
//! the following characteristics/constraints:
//!
//! * It must have a finite amount of possible states and moves
//! * No probability must be involved in state transitions
//! * It must be reasonably "sized" (in terms of number of unique states)
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

/// This is a list of the games implemented in GamesmanNova, by their names.
pub const IMPLEMENTED_GAMES: [&str; 1] = ["zero-by"];

/* GAME MODULE DECLARATIONS */

/* CYCLIC GAMES */

/* TIERED GAMES */

/* ACYCLIC GAMES */

/// A simple game about taking turns removing different amounts of elements
/// from a set of different sizes until there are none left.
pub mod zero_by;

/* TREE GAMES */

/* OTHER GAMES */

/* SHARED BEHAVIOR */

/// Factored-out behavior common to multiple game implementations.
pub mod utils;
