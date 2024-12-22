//! # Game Data Models Module
//!
//! Provides definitions for types used in game interfaces.

/// Unique identifier for a player in a game.
pub type Player = usize;

/// Unique identifier of a subset of states of a game.
pub type Partition = u64;

/// Count of the number of states in a game.
pub type StateCount = u64;

/// Count of the number of players in a game.
pub type PlayerCount = Player;
