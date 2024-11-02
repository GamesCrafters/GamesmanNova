//! # Solver Data Models Module
//!
//! Provides definitions for types used in solver implementations.

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
    Lose = 0,
    Draw = 1,
    Tie = 2,
    Win = 3,
}
