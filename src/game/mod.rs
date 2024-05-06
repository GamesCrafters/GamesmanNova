//! # Game Implementations Module
//!
//! The `games` module includes implementations for games intended to be
//! solved. To be able to solve a game with GamesmanNova, it must satisfy
//! the following characteristics/constraints:
//! * It must be reasonably "sized" (number of equivalent states).
//! * It must have states which can be efficiently represented.
//!
//! #### Authorship
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)
//! - Ishir Garg, 4/1/2024 (ishirgarg@berkeley.edu)

use anyhow::Result;

use crate::{
    interface::{IOMode, SolutionMode},
    model::{
        database::Identifier,
        game::{State, DEFAULT_STATE_BYTES},
    },
};

/* UTILITY MODULES */

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod test;

mod error;
mod util;

/* IMPLEMENTED GAMES */

pub mod crossteaser;
pub mod zero_by;

/* DEFINITIONS */

/// Contains useful data about a game, intended to provide users of the program
/// information they can use to understand the output of solving algorithms,
/// in addition to specifying game variants.
pub struct GameData {
    /* INSTANCE */
    /// The variant string used to initialize the `Game` instance which returned
    /// this `GameData` object from its `info` associated method.
    pub variant: String,

    /* GENERAL */
    /// Known name for the game. This should return a string that can be used as
    /// a command-line argument to the CLI endpoints which require a game name
    /// as a target (e.g. `nova solve <TARGET>`).
    pub name: &'static str,
    /// The names of the people who implemented the game listed out, optionally
    /// including their contact. For example: "John Doe <john@rust-lang.org>,
    /// Ricardo L. <ricardo@go-lang.com>, Quin Bligh".
    pub authors: &'static str,
    /// General introduction to the game's rules and setup, including any facts
    /// that are interesting about it.
    pub about: &'static str,

    /* VARIANTS */
    /// Explanation of how to use strings to communicate which variant a user
    /// wishes to play to the game's implementation.
    pub variant_protocol: &'static str,
    /// Regular expression pattern that all variant strings must match.
    pub variant_pattern: &'static str,
    /// Default variant string to be used when none is specified.
    pub variant_default: &'static str,

    /* STATES */
    /// Explanation of how to use a string to encode a game state.
    pub state_protocol: &'static str,
    /// Regular expression pattern that all state encodings must match.
    pub state_pattern: &'static str,
    /// Default state encoding to be used when none is specified.
    pub state_default: &'static str,
}

/* INTERFACES */

/// Defines miscellaneous behavior of a deterministic economic game object. Note
/// that player count is arbitrary; puzzles are semantically one-player games,
/// although they are more alike to optimization problems than cases of games.
pub trait Game {
    /// Returns `Result::Ok(Self)` if the specified `variant` is not malformed.
    /// Otherwise, returns a `Result::Err(String)` containing a text string
    /// explaining why it could not be parsed.
    fn new(variant: Option<String>) -> Result<Self>
    where
        Self: Sized;

    /// Returns an ID unique to this game. The return value should be consistent
    /// across calls from the same game and variant, but differing across calls
    /// from different games and variants. As such, it can be thought of as a
    /// string hash whose input is the game and variant (although it is not at
    /// all necessary that it conforms to any measure of hashing performance).
    fn id(&self) -> Identifier;

    /// Returns useful information about the game, such as the type of game it
    /// is, who implemented it, and an explanation of how to specify different
    /// variants for initialization.
    fn info(&self) -> GameData;

    /// Runs a solving algorithm which consumes the callee, generating side
    /// effects specified by the `mode` parameter. This should return an error
    /// if solving the specific game variant is not supported (among other
    /// possibilities for an error), and a unit type if everything goes per
    /// specification. See `IOMode` for specifics on intended side effects.
    fn solve(&self, mode: IOMode, method: SolutionMode) -> Result<()>;
}

/// Provides a way to retrieve a unique starting state from which to begin a
/// traversal, and a way to tell when a traversal can no longer continue from
/// a state. This does not necessarily imply that the underlying structure being
/// traversed over is finite; just that there exist finite traversals over it.
/// Generic over a state type **S**.
///
/// ## Explanation
///
/// In the example of games, there often exist ways to arrange their elements
/// in a way that unexpectedly invalidates game state. For example, there is no
/// valid game of chess with no kings remaining on the board. However, the most
/// intuitive implementations of `Transition` interfaces would not bat an eye at
/// this, and would simply return more states without any kings (this is one of
/// the more obvious examples of an invalid state, but there are subtler ones).
///
/// In addition, not all valid states may be reachable from other valid states.
/// For example, the empty board of Tic Tac Toe is not reachable from a board
/// with any number of pieces on the board. In some games, though, these states
/// become valid by simply changing the starting state (which is within the
/// realm of game variants). For example, in the game 10 to 0 by 1 or 3, it is
/// not valid to have a state of 8, but it becomes valid when the starting state
/// is made to be 11. A similar line of reasoning applies to end states.
///
/// These facts motivate that the logic which determines the starting and ending
/// states of games should be independent of the logic that transitions from
/// valid states to other valid states.
pub trait Bounded<const B: usize = DEFAULT_STATE_BYTES> {
    /// Returns the starting state of the underlying structure. This is used to
    /// deterministically initialize a traversal.
    fn start(&self) -> State<B>;

    /// Returns true if and only if there are no possible transitions from the
    /// provided `state`. Inputting an invalid `state` is undefined behavior.
    fn end(&self, state: State<B>) -> bool;
}

/// Defines behavior to encode and decode a state type **S** to and from a
/// `String`. This is related to the `GameData` object, which should contain
/// information about how game states can be represented using a string.
///
/// ## Explanation
///
/// Efficient game state hashes are rarely intuitive to understand due to being
/// highly optimized. Providing a way to transform them to and from a string
/// gives a representation that is easier to understand. This, in turn, can be
/// used throughout the project's interfaces to do things like fast-forwarding
/// a game to a user-provided state, providing readable debug output, etc.
///
/// Note that this is not supposed to provide a "fancy" printable game board
/// drawing; a lot of the utility obtained from implementing this interface is
/// having access to understandable yet compact game state representations. As
/// a rule of thumb, all strings should be single-lined and have no whitespace.
pub trait Codec<const B: usize = DEFAULT_STATE_BYTES> {
    /// Transforms a string representation of a game state into a type **S**.
    /// The `string` representation should conform to the `state_protocol`
    /// specified in the `GameData` object returned by `Game::info`. If it does
    /// not, an error containing a message with a brief explanation on what is
    /// wrong with `string` should be returned.
    fn decode(&self, string: String) -> Result<State<B>>;

    /// Transforms a game state type **S** into a string representation. The
    /// string returned should conform to the `state_protocol` specified in the
    /// `GameData` object returned by `Game::info`. If the `state` is malformed,
    /// this function should panic with a useful debug message. No two `state`s
    /// should return the same string representation (ideally).
    fn encode(&self, state: State<B>) -> String;
}

/// Provides a way to fast-forward a game state from its starting state (as
/// defined by `Bounded::start`) to a future state by playing a sequence of
/// string-encoded state transitions one after another. Generic over a state
/// type **S**.
///
/// # Explanation
///
/// For certain purposes, it is useful to skip a small or big part of a game's
/// states to go straight to exploring a subgame of interest, or because the
/// game is simply too large to explore in its entirety. In order to skip to
/// this part of a game, a valid state in that subgame must be provided.
///
/// Since it is generally impossible to verify that a given state is reachable
/// from the start of a game, it is necessary to demand a sequence of states
/// that begin in a starting state and end in the desired state, such that each
/// transition between states is valid per the game's ruleset.
pub trait Forward<const B: usize = DEFAULT_STATE_BYTES>
where
    Self: Bounded<B> + Codec<B>,
{
    /// Advances the game's starting state to the last state in `history`. All
    /// all of the `String`s in `history` must conform to the `state_protocol`
    /// defined in the `GameData` object returned by `info`. The states in
    /// `history` should be verified by ensuring that the following is true:
    ///
    /// - `history[0]` is the start state specified by the game variant.
    /// - The set `transition(history[i])` contains `history[i + 1]`.
    ///
    /// If these conditions are not satisfied, this function should return a
    /// useful error containing information about why the provided `history`
    /// is not possible for the game variant. Otherwise, it should mutate `self`
    /// to have a starting state whose string encoding is `history.pop()`.
    fn forward(&mut self, history: Vec<String>) -> Result<()>;
}

/// TODO
pub trait Transition<const B: usize = DEFAULT_STATE_BYTES> {
    /// Given a `state` at time `t`, returns all states that are possible at
    /// time `t + 1`. This should only guarantee that if `state` is feasible and
    /// not an end state, then all returned states are also feasible; therefore,
    /// inputting an invalid or end `state` is undefined behavior. The order of
    /// the values returned is insignificant.
    fn prograde(&self, state: State<B>) -> Vec<State<B>>;

    /// Given a `state` at time `t`, returns all states that are possible at
    /// time `t - 1`. This should only guarantee that if `state` is feasible,
    /// then all returned states are also feasible; therefore, inputting an
    /// invalid `state` is undefined behavior. The order of the values returned
    /// is insignificant.
    fn retrograde(&self, state: State<B>) -> Vec<State<B>>;
}
