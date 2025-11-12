//! # Game Implementations
//!
//! TODO

use anyhow::Result;
use clap::ValueEnum;

use std::fmt::Display;
use std::ops::Not;

use crate::error::SolverError;
use crate::frontend::GameAttribute;
use crate::game::traits::ClassicGame;
use crate::game::traits::ClassicPuzzle;
use crate::game::traits::IntegerUtility;
use crate::game::traits::SimpleUtility;

/* SUBMODULES */

pub mod traits;
pub mod util;

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

/* IMPLEMENTATIONS */

impl GameData {
    pub fn find(&self, attribute: GameAttribute) -> &str {
        match attribute {
            GameAttribute::VariantProtocol => self.variant_protocol,
            GameAttribute::VariantPattern => self.variant_pattern,
            GameAttribute::VariantDefault => self.variant_default,
            GameAttribute::StateProtocol => self.state_protocol,
            GameAttribute::StateDefault => self.state_default,
            GameAttribute::StatePattern => self.state_pattern,
            GameAttribute::Authors => self.authors,
            GameAttribute::About => self.about,
            GameAttribute::Name => self.name,
        }
    }
}

/* IMPL TRAIT FOR TYPE */

// All N-player simple-utility games are also N-player integer-utility games.
impl<const N: PlayerCount, const B: usize, G> IntegerUtility<N, B> for G
where
    G: SimpleUtility<N, B>,
{
    fn utility(&self, state: &State<B>) -> [IUtility; N] {
        let sutility = self.utility(state);
        let mut iutility = [0; N];
        iutility
            .iter_mut()
            .enumerate()
            .for_each(|(i, u)| *u = IUtility::from(sutility[i]) - 1);

        iutility
    }
}

// All 2-player zero-sum games are also 2-player simple-utility games.
impl<const B: usize, G> SimpleUtility<2, B> for G
where
    G: ClassicGame<B>,
{
    fn utility(&self, state: &State<B>) -> [SUtility; 2] {
        let mut sutility = [SUtility::Tie; 2];
        let utility = self.utility(state);
        let turn = self.turn(state);
        let them = (turn + 1) % 2;
        sutility[them] = !utility;
        sutility[turn] = utility;
        sutility
    }
}

// All puzzles are also 1-player simple-utility games.
impl<const B: usize, G> SimpleUtility<1, B> for G
where
    G: ClassicPuzzle<B>,
{
    fn utility(&self, state: &State<B>) -> [SUtility; 1] {
        [self.utility(*state)]
    }
}

/* IMPL EXTERNAL TRAIT */

impl TryFrom<IUtility> for SUtility {
    type Error = SolverError;

    fn try_from(v: IUtility) -> Result<Self, Self::Error> {
        match v {
            _ if v == SUtility::Lose as i64 => Ok(SUtility::Lose),
            _ if v == SUtility::Tie as i64 => Ok(SUtility::Tie),
            _ if v == SUtility::Win as i64 => Ok(SUtility::Win),
            _ => Err(SolverError::InvalidConversion {
                input_t: "Integer Utility".into(),
                output_t: "Simple Utility".into(),
                hint:
                    "Down-casting from integer to simple utility values is not \
                    stable, and relies on the internal representation used for \
                    simple utility values."
                        .into(),
            }),
        }
    }
}

impl TryFrom<i8> for SUtility {
    type Error = SolverError;

    fn try_from(v: i8) -> Result<Self, Self::Error> {
        match v {
            _ if v == SUtility::Lose as i8 => Ok(SUtility::Lose),
            _ if v == SUtility::Tie as i8 => Ok(SUtility::Tie),
            _ if v == SUtility::Win as i8 => Ok(SUtility::Win),
            _ => Err(SolverError::InvalidConversion {
                input_t: "i8".into(),
                output_t: "Simple Utility".into(),
                hint: "Down-casting from integer to simple utility values \
                    is not stable, and relies on the internal representation \
                    used for simple utility values."
                    .into(),
            }),
        }
    }
}

impl From<SUtility> for IUtility {
    fn from(v: SUtility) -> Self {
        match v {
            SUtility::Lose => -1,
            SUtility::Tie => 0,
            SUtility::Win => 1,
        }
    }
}

impl Not for SUtility {
    type Output = SUtility;
    fn not(self) -> Self::Output {
        match self {
            SUtility::Lose => SUtility::Win,
            SUtility::Win => SUtility::Lose,
            SUtility::Tie => SUtility::Tie,
        }
    }
}

impl Display for GameAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            GameAttribute::VariantProtocol => "variant-protocol",
            GameAttribute::VariantPattern => "variant-pattern",
            GameAttribute::VariantDefault => "variant-default",
            GameAttribute::StateProtocol => "state-protocol",
            GameAttribute::StateDefault => "state-default",
            GameAttribute::StatePattern => "state-pattern",
            GameAttribute::Authors => "authors",
            GameAttribute::About => "about",
            GameAttribute::Name => "name",
        };
        write!(f, "{content}")
    }
}
