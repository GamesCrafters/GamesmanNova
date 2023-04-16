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

/// Trait declarations for different marked classes of games.
pub mod archetypes;

/* GAME MODULE DECLARATIONS */

/* Macro explanations:
//
// The first macro looks in this directory (games/) and expands to a collection of
// module definitions as follows:
//
// ```
// pub mod game_1;
// pub mod game_2;
// ...
// pub mod game_n;
// ```
//
// The second macro does the same thing, but instead of generating module
// definitions it automatically creates a constant list of their names
// as follows:
//
// ```
// pub const GAME_LIST: [&str; n] = [
//    game_1,
//    ...
//    game_n,
// ];
// ```
//
// The second one was handmade, which is why it's poorly documented.
*/
dirmod::all!(default pub; except archetypes);
list_modules::here!("src/games/");
