#![warn(missing_docs)]

//! # GamesmanNova Core Library
//!
//! `core` is a collection of analyzers, databases, and solvers which can be
//! applied to any deterministic finite-state abstract strategy game through
//! common abstract interfaces.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

/* MODULES */

/// Tools for game state graph aggregation, reduction, and visualization.
pub mod analyzers;

/// Database formats suited for storing different kinds of solver results.
pub mod databases;

/// Types of games based on their fundamental characteristics.
pub mod archetypes;

/// Algorithms for solving finite-state deterministic games.
pub mod solvers;

/* TYPES */

/// Encodes the configuration of a game in a string, which allows game
/// implementations to set themselves up differently depending on its contents.
pub type Variant = String;

/// Encodes the state of a game in a 64-bit unsigned integer. This also
/// sets a limiting upper bound on the amount of possible non-equivalent states
/// that can be achieved in a game.
pub type State = u64;

/// Encodes a possible move of a game in a 32-bit unsigned integer. This also
/// sets a limit on the amount of possible distinct moves that can be made in
/// a game.
pub type Move = u32;

/* ENUMERATIONS */

/// Indicates the value of a game state according to the game's rules. Contains
/// remoteness information (how far away a state is from its corresponding
/// terminal state).
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Value {
    /// Indicates that a player has won.
    Win(u8),
    /// Indicates that a player has lost.
    Lose(u8),
    /// Indicates that the game is a tie.
    Tie(u8),
}
