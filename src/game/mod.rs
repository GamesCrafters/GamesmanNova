#![forbid(unsafe_code)]
//! # Game Module
//!
//! TODO

use anyhow::{Context, Result};
use clap::ValueEnum;
use once_cell::sync::OnceCell;
use sqlx::SqlitePool;

/* UTILITY MODULES */

#[cfg(test)]
mod test;

pub mod util;
pub mod error;

/* GAME MODULES */

#[cfg(test)]
pub mod mock;

pub mod zero_by;
pub mod crossteaser;

/* TYPES */

/// The default number of bytes used to encode states.
pub const DEFAULT_STATE_BYTES: usize = 8;

/// Unique identifier of a particular state in a game.
pub type State<const B: usize = DEFAULT_STATE_BYTES> = [u8; B];

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

/* SINGLETONS */

/// TODO
pub static DB: OnceCell<SqlitePool> = OnceCell::new();

/* DEFINITIONS */

// Specifies the game offerings available through all interfaces.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum GameModule {
    /// Grid-like 3d puzzle with rotating pieces.
    Crossteaser,

    /// Abstract game played over sets of items.
    ZeroBy,
}

/// Contains useful data about a game.
///
/// The information here is intended to provide users of the program information
/// they can use to understand the output of feature extractors, in addition to
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

/* EXTRACTION INTERFACES */

pub trait Information {
    /// TODO
    fn info() -> GameData;
}

/* IMPLICIT GRAPH INTERFACE */

pub trait Implicit<const B: usize = DEFAULT_STATE_BYTES> {
    /// TODO
    fn adjacent(&self, state: State<B>) -> Vec<State<B>>;

    /// TODO
    fn source(&self) -> State<B>;

    /// TODO
    fn sink(&self, state: State<B>) -> bool;
}

pub trait Transpose<const B: usize = DEFAULT_STATE_BYTES> {
    /// TODO
    fn adjacent(&self, state: State<B>) -> Vec<State<B>>;
}

pub trait Composite<const B: usize = DEFAULT_STATE_BYTES> {
    /// TODO
    fn partition(&self, state: State<B>) -> Partition;

    /// TODO
    fn size(&self, partition: Partition) -> StateCount;
}

/* UTILITY INTEFACES */

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
    /// ```
    /// use crate::game::zero_by;
    /// let default_variant = zero_by::Session::new();
    /// assert_eq!(
    ///     default_variant.decode("10-0".into())?,
    ///     default_variant.start()
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
    /// ```
    /// use crate::game::zero_by;
    /// let default_variant = zero_by::Session::new();
    /// assert_eq!(
    ///     default_variant.encode(default_variant.start())?,
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
    /// ```
    /// use crate::game::zero_by;
    ///
    /// let default = zero_by::Session::new();
    /// assert_ne!(default.encode(default.start())?, state);
    ///
    /// let state = "100-0".into();
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

    /// Returns a string representing the underlying game variant.
    ///
    /// This does not provide a certain way of differentiating between the
    /// starting state of the game (see [`Bounded::start`] for this), but it
    /// does provide a sufficient identifier of the game's structure.
    ///
    /// # Example
    ///
    /// Consider the following example on a game of [`zero_by`], which has the
    /// default variant of `"2-10-1-2"`:
    ///
    /// ```
    /// use crate::game::zero_by;
    ///
    /// let variant = "3-100-3-4".into();
    /// let default_variant = zero_by::Session::new();
    /// assert_eq!(default_variant.variant(), "2-10-1-2".into());
    ///
    /// let custom_variant = session.into_variant(variant.clone())?;
    /// assert_eq!(custom_variant.variant(), variant);
    /// ```
    fn variant_string(&self) -> Variant;
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
    /// ```
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
    /// # Example
    ///
    /// This can be useful for skipping a significant amount of computation in
    /// the process of performing subgame analysis. Requires an implementation
    /// of [`Forward::set_verified_start`] to ultimately change the starting
    /// state after `history` is verified.
    ///
    /// Using the game [`zero_by`] with a default state of `"10-0"`:
    ///
    /// ```
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
    /// Here are some of the reasons this could fail:
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
