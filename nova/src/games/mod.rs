//! # Game Implementations Module
//!
//! The `games` crate includes implementations for games intended to be
//! solved. To be able to solve a game, with GamesmanNova, it must satisfy
//! the following characteristics/constraints:
//!
//! * It must have a finite amount of possible states and moves
//! * No probability must be involved in state transitions
//! * It must be reasonably "sized" (in terms of number of unique states)
//!
//! This module contains overarching classes of games in the form of traits.
//! In general, these are not always mutually exclusive; it is the case that
//! they share many of the same fundamental characteristics. However, they are
//! separated in terms of the opportunity that is awarded by considering what
//! makes each one of them unique.
//!
//! This is mainly motivated by novel approaches to solving algorithms, but the
//! categories can be extended arbitrarily much. For example, different kinds
//! of interface to GamesmanNova might want to consider these categories
//! differently, or database implementations might be different when
//! considering the underlying structure of the game involved.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::models::{Solver, State, Value, Variant};

/* INTEGRATION AUTOMATION PROCEDURAL MACROS */

/* Looks in this directory (games/) and expands to a collection of
 * module definitions as follows:
 *
 * ```
 * pub mod game_1;
 * pub mod game_2;
 * ...
 * pub mod game_n;
 * ```
 */
dirmod::all!(default pub);

/* Does the same thing, but instead of generating module definitions it
 * automatically creates a constant list of their names as follows:
 *
 * ```
 * pub const LIST: [&str; n] = [
 *    "game_1",
 *    "game_2",
 *    ...
 *    "game_n",
 * ];
 * ```
 */
list_modules::here!("src/games/");

/* BASE CONSTRUCTS */

/// A generic deterministic finite-state game or puzzle.
pub trait Game
{
    /// Allows for the specification of a game variant and the initialization of
    /// a game's internal representation.
    fn initialize(variant: Option<Variant>) -> Self
    where
        Self: Sized;

    /// Returns an ID unique to this game and consistent across calls from the
    /// same game variant.
    fn id(&self) -> String;

    /// Returns useful information about the game.
    fn info(&self) -> GameInformation;
}

/// Contains useful information about a game.
pub struct GameInformation
{
    /// Known name for the game.
    pub name: String,

    /// The people who implemented the game.
    pub author: String,

    /// General introduction to the game.
    pub about: String,

    /// Explanation of how to use strings to communicate which variant a user
    /// wishes to play to the implementation.
    pub variant_protocol: String,

    /// Regular expression pattern that all variant strings must match.
    pub variant_pattern: String,

    /// Default variant string to be used when none is specified.
    pub variant_default: String,
}

/// Indicates that a game is solvable, and offers a function to retrieve
/// the solvers that can solve the game.
pub trait Solvable
where
    Self: Game,
{
    /// Returns the set of possible states one move away from `state`.
    fn adjacent(&self, state: State) -> Vec<State>;

    /// Returns `None` if the state is non-terminal, and a `Value` otherwise.
    fn value(&self, state: State) -> Option<Value>;

    /// Returns the game's starting state.
    fn start(&self) -> State;

    /// Returns all the solvers available to solve the game in order of
    /// overall efficiency, including their interface names. The option
    /// to choose a default solver in the implementation of this function
    /// is allowed by making one of them mapped to `None`, as opposed to
    /// `Some(String)`.
    fn solvers(&self) -> Vec<(String, Solver<Self>)>;
}

/* SOLVING MARKERS */

/// Indicates that a game's state graph can be partitioned into independent
/// connected components and solved taking advantage of this.
pub trait TierSolvable
where
    Self: Solvable,
{
}

/// Indicates that a game is solvable in a generally inefficient manner.
pub trait CyclicallySolvable
where
    Self: Solvable,
{
}

/// Indicates that a game is solvable using methods only available to games
/// whose state graphs are acyclic (which includes tree games).
pub trait AcyclicallySolvable
where
    Self: Solvable,
{
}

/// Indicates that a game is solvable using methods only available to games
/// with unique move paths to all states.
pub trait TreeSolvable
where
    Self: Solvable,
{
}
