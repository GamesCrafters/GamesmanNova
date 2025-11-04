//! # Game Models
//!
//! TODO

use clap::ValueEnum;

/* SUBMODULES */

#[cfg(test)]
pub mod mock;
pub mod zero_by;

/* TYPE ALIASES */

/// Unique identifier of a particular state in a game.
pub type State<const B: usize = DEFAULT_STATE_BYTES> = [u8; B];

/// Indicates the number of outoing edges that exist from a given game state.
pub type Degree = u64;

/// An element of a partition of a game graph, where the partition elements are
/// structured as a DAG as seen through the graph cuts they induce.
pub type Component = u64;

/// Indicates the number of choices that players have to make to reach a
/// terminal state in a game under perfect play. For drawing positions,
/// indicates the number of choices players can make to bring the game to a
/// state which can transition to a non-drawing state.
pub type Remoteness = u64;

/// String encoding some specific game's variant.
pub type Variant = String;

/// Unique identifier for a player in a game.
pub type Player = usize;

/// Count of the number of players in a game.
pub type PlayerCount = Player;

/// A discrete measure of how "good" an outcome is for a given player. Positive
/// values indicate an overall gain from having played the game, and negative
/// values are net losses.
pub type IUtility = i64;

/* CONSTANTS */

/// The default number of bytes used to encode states.
pub const DEFAULT_STATE_BYTES: usize = 8;

/* ENUMERATIONS */

// Specifies the game offerings available through all interfaces.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum GameModule {
    /// Abstract game played over sets of items.
    ZeroBy,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum SUtility {
    Lose = 0,
    Tie = 1,
    Win = 2,
}

#[derive(Clone, Copy)]
pub enum UtilityType {
    Integer,
    Simple,
}

/* STRUCTURES */

/// Contains useful data about a game.
///
/// The information here is intended to provide users of the program information
/// they can use to understand the output of solving algorithms, in addition to
/// specifying formats/protocols for communicating with game implementations,
/// and providing descriptive error outputs. See [`Information::info`] for how
/// to expose this information.
///
/// # Example
///
/// In the case of the sequential game [`zero_by`]:
///
/// ```none
/// * Name: "zero-by"
/// * Authors: "John Doe <john@email.com>, Jane Doe <jane@email.com>"
/// * About: "Zero By is a multiplayer zero-sum game where N players ..."
/// * Variant protocol: "Three or more dash-separated strings, where..."
/// * Variant pattern: r"^[1-9]\d*(?:-[1-9]\d*)+$"
/// * Variant default: "2-10-1-2"
/// * State protocol: "The state string should be two dash-separated ..."
/// * State pattern: r"^\d+-\d+$"
/// * State default: "10-0"
/// ```
pub struct GameData {
    /* GENERAL */
    /// Known name for the game. This should return a string that can be used
    /// in the command-line as an argument to the CLI endpoints which require a
    /// name as a game (e.g. `nova solve <TARGET>`).
    pub name: &'static str,

    /// The names of people who implemented the game listed out, optionally
    /// including their contact. For example: "John Doe <john@rust-lang.org>,
    /// Ricardo L. <ricardo@go-lang.com>, Quin Bligh".
    pub authors: &'static str,

    /// General introduction to the game's rules, setup, etc., including any
    /// facts that are noteworthy about it.
    pub about: &'static str,

    /* VARIANTS */
    /// Explanation of how to use strings to communicate which variant a user
    /// wishes to provide to the game's implementation.
    pub variant_protocol: &'static str,

    /// Regular expression pattern that all variant strings must match.
    pub variant_pattern: &'static str,

    /// Default variant string to be used when none is specified.
    pub variant_default: &'static str,

    /* STATES */
    /// Explanation of how to use a string to encode an abstract state.
    pub state_protocol: &'static str,

    /// Regular expression pattern that all state encodings must match.
    pub state_pattern: &'static str,

    /// Default state encoding to be used when none is specified.
    pub state_default: &'static str,
}
