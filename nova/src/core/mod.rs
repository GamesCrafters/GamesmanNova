//! # Core Behavior Module
//!
//! `core` is a collection of analyzers, databases, and solvers which can be
//! applied to any deterministic finite-state abstract strategy game through
//! common abstract interfaces.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

/* MODULES */

pub mod analyzers;
pub mod databases;
pub mod solvers;

/* TYPES */

/// Encodes the configuration of a game in a string, which allows game
/// implementations to set themselves up differently depending on its contents.
/// The protocol used to map a variant string to a specific game setup is
/// decided by the implementation of a game, so reading game-specific
/// documentation will be necessary to porperly form a variant string.
pub type Variant = String;

/// Encodes the state of a game in a 64-bit unsigned integer. This also
/// sets a limiting upper bound on the amount of possible non-equivalent states
/// that can be achieved in a game.
pub type State = u64;

/// The signature of a function which can solve a game, taking in the game,
/// and parameters read and write.
pub type SolverFn<G> = fn(&G, bool, bool) -> Value;

/* ENUMERATIONS */

/// Indicates the value of a game state according to the game's rules. Contains
/// remoteness information (how far away a state is from its corresponding
/// terminal state).
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Value {
    /// Indicates that a player has won.
    Win(u32),
    /// Indicates that a player has lost.
    Lose(u32),
    /// Indicates that the game is a tie.
    Tie(u32),
}
