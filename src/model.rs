#![allow(dead_code)]
//! # Data Models Module
//!
//! This module contains centralized definitions for custom data types used
//! throughout the project.
//!
//! #### Authorship
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)
//! - Ishir Garg, 4/1/2024 (ishirgarg@berkeley.edu)

/// # Game Data Models Module
///
/// Provides definitions for types used in game interfaces.
///
/// #### Authorship
/// - Max Fierro, 4/30/2024 (maxfierro@berkeley.edu)
pub mod game {

    use bitvec::{array::BitArray, order::Msb0};

    /// The default number of bytes used to encode states.
    pub const DEFAULT_STATE_BYTES: usize = 8;

    /// Unique identifier of a particular state in a game.
    pub type State<const B: usize = DEFAULT_STATE_BYTES> =
        BitArray<[u8; B], Msb0>;

    /// Unique identifier for a player in a game.
    pub type Player = usize;

    /// Unique identifier of a subset of states of a game.
    pub type Partition = u64;

    /// Count of the number of states in a game.
    pub type StateCount = u64;

    /// Count of the number of players in a game.
    pub type PlayerCount = Player;
}

/// # Solver Data Models Module
///
/// Provides definitions for types used in solver implementations.
///
/// #### Authorship
/// - Max Fierro, 5/5/2024 (maxfierro@berkeley.edu)
pub mod solver {
    /// Indicates the "depth of draw" which a drawing position corresponds to.
    /// For more information, see [this whitepaper](TODO). This value should be
    /// 0 for non-drawing positions.
    pub type DrawDepth = u64;

    /// Indicates the number of choices that players have to make to reach a
    /// terminal state in a game under perfect play. For drawing positions,
    /// indicates the number of choices players can make to bring the game to a
    /// state which can transition to a non-drawing state.
    pub type Remoteness = u64;

    /// Please refer to [this](https://en.wikipedia.org/wiki/Mex_(mathematics)).
    pub type MinExclusion = u64;

    /// A measure of how "good" an outcome is for a given player in a game.
    /// Positive values indicate an overall gain from having played the game,
    /// and negative values are net losses. The metric over abstract utility is
    /// subjective.
    pub type RUtility = f64;

    /// A discrete measure of how "good" an outcome is for a given player.
    /// Positive values indicate an overall gain from having played the game,
    /// and negative values are net losses. The metric over abstract utility is
    /// subjective.
    pub type IUtility = i64;

    /// A simple measure of hoe "good" an outcome is for a given player in a
    /// game. The specific meaning of each variant can change based on the game
    /// in consideration, but this is ultimately an intuitive notion.
    #[derive(Clone, Copy)]
    pub enum SUtility {
        WIN = 0,
        LOSE = 1,
        DRAW = 2,
        TIE = 3,
    }
}

/// # Database Data Models Module
///
/// Provides definitions for types used in database interfaces.
///
/// #### Authorship
/// - Max Fierro, 4/30/2024 (maxfierro@berkeley.edu)
pub mod database {

    use bitvec::order::Msb0;
    use bitvec::slice::BitSlice;

    /// A generic number used to differentiate between objects.
    pub type Identifier = u64;

    /// The type of a raw sequence of bits encoding a database value associated
    /// with a key, backed by a [`BitSlice`] with [`u8`] big-endian storage.
    pub type Value = BitSlice<u8, Msb0>;

    /// The type of a database key per an implementation of [`KVStore`].
    pub type Key = BitSlice<u8, Msb0>;
}
