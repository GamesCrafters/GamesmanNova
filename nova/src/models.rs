//! # Data Models Module
//!
//! This module contains centralized definitions for custom datatypes used
//! throughout the project.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

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
pub type Solver<G> = fn(&G, bool, bool) -> Value;

/* ENUMERATIONS */

/// Indicates the value of a game state according to the game's rules. Contains
/// remoteness information (how far away a state is from its corresponding
/// terminal state).
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Value
{
    /// Indicates that a player has won.
    Win(u32),

    /// Indicates that a player has lost.
    Lose(u32),

    /// Indicates that the game is a tie.
    Tie(u32),
}
