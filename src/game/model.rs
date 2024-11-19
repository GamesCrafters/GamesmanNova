//! # Game Data Models Module
//!
//! Provides definitions for types used in game interfaces.

use bitvec::{array::BitArray, order::Msb0};
use clap::ValueEnum;

/// The default number of bytes used to encode states.
pub const DEFAULT_STATE_BYTES: usize = 8;

/// Unique identifier of a particular state in a game.
pub type State<const B: usize = DEFAULT_STATE_BYTES> = BitArray<[u8; B], Msb0>;

/// String encoding some specific game's variant.
pub type Variant = String;

/// Unique identifier for a player in a game.
pub type Player = usize;

/// Unique identifier of a subset of states of a game.
pub type Partition = u64;

/// Count of the number of states in a game.
pub type StateCount = u64;

/// Count of the number of players in a game.
pub type PlayerCount = Player;

// Specifies the game offerings available through all interfaces.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum GameModule {
    ZeroBy,
}
