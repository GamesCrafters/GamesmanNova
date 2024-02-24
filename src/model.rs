#![allow(dead_code)]
//! # Data Models Module
//!
//! This module contains centralized definitions for custom data types used
//! throughout the project.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

/* PRIMARY TYPES */

/// Encodes the state of a game in a 64-bit unsigned integer. This also
/// sets a limiting upper bound on the amount of possible non-equivalent states
/// that can be achieved in a game.
pub type State = u64;

/// Expresses whose turn it is in a game, where every player is assigned to a
/// different integer. Note that the type imposes a natural (but unknown)
/// limitation to player count that is dependent on the target architecture.
pub type Turn = usize;

/* ATTRIBUTE TYPES */

/// A measure of how "good" an outcome is for a given player in a game. Positive
/// values indicate an overall gain from having played the game, and negative
/// values are net losses. The metric over abstract utility is subjective.
pub type Utility = i64;

/// Indicates the "depth of draw" which a drawing position corresponds to. For
/// more information, see [this whitepaper](TODO). This value should be 0 for
/// non-drawing positions.
pub type DrawDepth = u64;

/// Indicates the number of choices that players have to make to reach a
/// terminal state in a game under perfect play. For drawing positions,
/// indicates the number of choices players can make to bring the game to a
/// state which can transition to a non-drawing state.
pub type Remoteness = u64;

/// Please refer to [this](https://en.wikipedia.org/wiki/Mex_(mathematics)).
pub type MinimumExcludedValue = u64;

/// The length of a database record in bytes under a specific schema.
pub type RecordLength = u32;

/* SECONDARY TYPES */

/// Used to count the number of states in a set. Although this has an identical
/// underlying type as `State`, it is semantically different, which is why it is
/// declared under a different type.
pub type StateCount = State;

/// Used to count the number of players in a game. Although this has a type that
/// is identical to `Turn`, it is semantically different, which is why it has
/// its own type declaration.
pub type PlayerCount = Turn;

/// Encodes an identifier for a given partition within the space of states of a
/// game. This is a secondary type because the maximum number of partitions is
/// the number of states itself.
pub type Partition = State;
