#![forbid(unsafe_code)]
//! # Game Module
//!
//! Contains definitions and blanket implementations that support sequential
//! games viewed as implicit grpahs. Special attention is paid to supporting
//! families of closely related games (variants).

use anyhow::Context;
use anyhow::Result;
use clap::ValueEnum;

/* UTILITY MODULES */

#[cfg(test)]
mod test;

pub mod util;
pub mod error;

/* GAME MODULES */

#[cfg(test)]
pub mod mock;

pub mod zero_by;
pub mod mnk;

/* TYPES */

/// The default number of bytes used to encode states.
pub const DEFAULT_STATE_BYTES: usize = 8;

/// Unique identifier of a particular state in a game.
pub type State<const B: usize = DEFAULT_STATE_BYTES> = [u8; B];

/// String encoding some specific game's variant.
pub type Variant = String;

/// Unique identifier for a player in a game.
pub type Player = usize;

/// Count of the number of players in a game.
pub type PlayerCount = Player;

/* DEFINITIONS */

// Specifies the game offerings available through all interfaces.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum GameModule {
    /// Abstract game played over sets of items.
    ZeroBy,

    /// Generalized version of Tic-Tac-Toe.
    Mnk,
}

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

/* INTERFACES */

pub trait Information {
    /// Returns useful information about the game family. See [`GameData`].
    fn info() -> GameData;
}

pub trait Implicit<const B: usize = DEFAULT_STATE_BYTES> {
    /// Returns the collection of states adjacent to `state` in this graph.
    ///
    /// The graph is assumed to be directed, such that calling this method on
    /// an element of its own output is not guaranteed to return the original
    /// state. An empty collection is used to denote a lack of neighbors.
    ///
    /// # Example
    ///
    /// Considering the sequential game [`zero_by`], where a player may choose
    /// remove either 1 or 2 elements from a pile of things:
    ///
    /// ```ignore
    /// use crate::game::zero_by;
    /// let session = zero_by::Session::new();
    ///
    /// // ignoring turn information for illustration only; "5 elements"
    /// let count = 5;
    ///
    /// // neighbors = [4, 3]; "4 elements, 3 elements"
    /// let neighbors = session.adjacent(count);
    /// ```
    ///
    /// # Panics
    ///
    /// If the implementation fails to decode the provided `state`, there are no
    /// behavior guarantees (this many or may not panic).
    fn adjacent(&self, state: State<B>) -> Vec<State<B>>;

    /// Returns one node within the implicit graph.  
    ///
    /// Since this is the first node to be explored when this interface is used,
    /// this is called the 'source' node. This does not mean it has an indegree
    /// of zero.
    ///
    /// # Example
    ///
    /// Considering the sequential game [`zero_by`], which begins with a state
    /// of 10 by default:
    ///
    /// ```ignore
    /// use crate::game::zero_by;
    /// let session = zero_by::Session::new();
    ///
    /// // ignoring turn information for illustration purposes
    /// assert_eq!(10, session.source());
    /// ```
    fn source(&self) -> State<B>;

    /// Returns true iff `state` has no outgoing edges in this graph.
    ///
    /// This is the source of truth for this condition. That is to say, it is
    /// considered incorrect for there to be a state for which `adjacent(state)`
    /// provides a non-empty collection, but where `sink(state)` is `false`.
    ///
    /// # Example
    ///
    /// Considering the sequential game [`zero_by`], which ends when there are
    /// no items left to play with:
    ///
    /// ```ignore
    /// use crate::game::zero_by;
    /// let session = zero_by::Session::new();
    ///
    /// // ignoring turn information for illustration purposes
    /// assert!(session.sink(0));
    /// ```
    fn sink(&self, state: State<B>) -> bool;
}

pub trait Codec<const B: usize = DEFAULT_STATE_BYTES> {
    /// Decodes a state [`String`] encoding into a bit-packed [`State<B>`].
    ///
    /// This function (and [`Codec::encode`]) effectively specifies a protocol
    /// for turning a [`String`] into a [`State<B>`]. See [`Information::info`]
    /// to make this protocol explicit.
    ///
    /// # Example
    ///
    /// Using the game [`zero_by`] with default state of `"10-0"`:
    ///
    /// ```ignore
    /// use crate::game::zero_by;
    /// let session = zero_by::Session::new();
    /// assert_eq!(
    ///     session.decode("10-0".into())?,
    ///     session.start()
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// Fails if `state` is detectably invalid or unreachable in the underlying
    /// game variant.
    fn decode(&self, string: String) -> Result<State<B>>;

    /// Encodes a game `state` into a compact string representation.
    ///
    /// The output representation is not designed to be space efficient. It is
    /// used for manual input/output. This function (and [`Codec::decode`])
    /// effectively specifies a protocol for translating a [`State<B>`] into
    /// a [`String`]. See [`Information::info`] to make this protocol explicit.
    ///
    /// # Example
    ///
    /// Using the game [`zero_by`] with a default state of `"10-0"`:
    ///
    /// ```ignore
    /// use crate::game::zero_by;
    /// let session = zero_by::Session::new();
    /// assert_eq!(
    ///     session.encode(session.start())?,
    ///     "10-0".into()
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// Fails if `state` is detectably invalid or unreachable in the underlying
    /// game variant.
    fn encode(&self, state: State<B>) -> Result<String>;
}

pub trait Variable {
    /// Initializes a version of the underlying game as the specified `variant`.
    ///
    /// A variant is a member of a family of games whose structure is very
    /// similar. It is convenient to be able to express this because it saves
    /// a lot of needless re-writing of game logic, while allowing for a lot
    /// of generality in game implementations.
    ///
    /// # Example
    ///
    /// Consider the following example on a game of [`zero_by`], which has a
    /// default starting state encoding of `"10-0"`:
    ///
    /// ```ignore
    /// use crate::game::zero_by;
    /// let state = "100-0".into();
    /// let default = zero_by::Session::new();
    /// assert_ne!(default.encode(default.start())?, state);
    ///
    /// let variant = zero_by::Session::variant("3-100-3-4".into())?;
    /// assert_eq!(variant.encode(variant.start())?, state);
    /// ```
    ///
    /// # Errors
    ///
    /// Fails if `variant` does not conform to the game's protocol of encoding
    /// variants as strings, or if the game does not support variants in the
    /// first place (but has a placeholder [`Variable`] implementation).
    fn variant(variant: Variant) -> Result<Self>
    where
        Self: Sized;
}

pub trait Forward<const B: usize = DEFAULT_STATE_BYTES>
where
    Self: Information + Codec<B> + Implicit<B> + Sized,
{
    /// Sets the game's starting state to a pre-verified `state`.
    ///
    /// This function is an auxiliary item for [`Forward::forward`]. While it
    /// needs to be implemented for [`Forward::forward`] to work, there should
    /// never be a need to call this directly from any other place. This would
    /// produce potentially incorrect behavior, as it is not possible to verify
    /// whether a state encoding is valid statically (in the general case).
    ///
    /// # Deprecated
    ///
    /// This function is marked deprecated to discourage direct usage, not
    /// because it is an actually deprecated interface item.
    ///
    /// # Example
    ///
    /// Using the game [`zero_by`] with a default state of `"10-0"`:
    ///
    /// ```ignore
    /// use crate::game::zero_by;
    ///
    /// let mut game = zero_by::Session::new();
    /// let start = game.decode("9-1".into())?;
    /// game.set_verified_start(start);
    ///
    /// assert_eq!(forwarded.encode(game.start)?, "9-1".into());
    /// ```
    #[deprecated(
        note = "This function should not be used directly; any modification of \
        initial states should be done through [`Forward::forward`], which is \
        fallible and provides verification for game states."
    )]
    fn set_verified_start(&mut self, state: State<B>);

    /// Advances the game's starting state to the last state in `history`,
    /// verifying that it is a valid traversal of the induced graph on this
    /// game variant.
    ///
    /// This can be useful for skipping a significant amount of computation in
    /// the process of performing subgame analysis. Requires an implementation
    /// of [`Forward::set_verified_start`] to ultimately change the starting
    /// state after `history` is verified.
    ///
    /// # Example
    ///
    /// Using the game [`zero_by`] with a default state of `"10-0"`:
    ///
    /// ```ignore
    /// use crate::game::zero_by;
    ///
    /// let mut game = zero_by::Session::new();
    /// let history = vec![
    ///     "10-0".into(),
    ///     "9-1".into(),
    ///     "8-0".into(),
    ///     "6-1".into(),
    /// ];
    ///
    /// let forwarded = game.forward(history)?;
    /// assert_eq!(forwarded.encode(forwarded.start())?, "6-1".into());
    /// ```
    ///
    /// # Errors
    ///
    /// Some reasons this could fail:
    /// * An invalid transition is made between subsequent states in `history`.
    /// * `history` begins at a state other than the variant's starting state.
    /// * The provided `history` transitions beyond a terminal state.
    /// * A state encoding in `history` is not valid.
    /// * `history` is empty.
    #[allow(deprecated)]
    fn forward(&mut self, history: Vec<String>) -> Result<()> {
        let to = util::verify_state_history(self, history)
            .context("Specified invalid state history.")?;

        self.set_verified_start(to);
        Ok(())
    }
}
