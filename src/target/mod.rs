#![forbid(unsafe_code)]
//! # Target Module
//!
//! This module provides interfaces and implementations for feature extraction
//! targets.

use anyhow::{Context, Result};
use clap::ValueEnum;

/* UTILITY MODULES */

#[cfg(test)]
mod test;

pub mod util;
pub mod error;

/* TARGET MODULES */

#[cfg(test)]
pub mod mock;

pub mod zero_by;
pub mod crossteaser;

/* TYPES */

/// The default number of bytes used to encode states.
pub const DEFAULT_STATE_BYTES: usize = 8;

/// Unique identifier of a particular state in a target.
pub type State<const B: usize = DEFAULT_STATE_BYTES> = [u8; B];

/// String encoding some specific target's variant.
pub type Variant = String;

/// Unique identifier for a player in a game.
pub type Player = usize;

/// Unique identifier of a subset of states of a game.
pub type Partition = u64;

/// Count of the number of states in a game.
pub type StateCount = u64;

/// Count of the number of players in a game.
pub type PlayerCount = Player;

/* DEFINITIONS */

// Specifies the target offerings available through all interfaces.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum TargetModule {
    /// Grid-like 3d puzzle with rotating pieces.
    Crossteaser,

    /// Abstract game played over sets of items.
    ZeroBy,
}

/// Contains useful data about an extraction target.
///
/// The information here is intended to provide users of the program information
/// they can use to understand the output of feature extractors, in addition to
/// specifying formats/protocols for communicating with target implementations,
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
pub struct TargetData {
    /* GENERAL */
    /// Known name for the target. This should return a string that can be used
    /// in the command-line as an argument to the CLI endpoints which require a
    /// name as a target (e.g. `nova solve <TARGET>`).
    pub name: &'static str,

    /// The names of people who implemented the target listed out, optionally
    /// including their contact. For example: "John Doe <john@rust-lang.org>,
    /// Ricardo L. <ricardo@go-lang.com>, Quin Bligh".
    pub authors: &'static str,

    /// General introduction to the target's rules, setup, etc., including any
    /// facts that are noteworthy about it.
    pub about: &'static str,

    /* VARIANTS */
    /// Explanation of how to use strings to communicate which variant a user
    /// wishes to provide to the target's implementation.
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
    /// Provides a way to retrieve useful information about a target for both
    /// internal and user-facing modules.
    ///
    /// The information included here should be broadly applicable to any
    /// variant of the underlying target type (hence why it is a static method).
    /// For specifics on the information to provide, see [`TargetData`].
    ///
    /// # Example
    ///
    /// Using the game [`zero_by`] as an example:
    ///
    /// ```
    /// use crate::game::zero_by;
    /// let game = zero_by::Session::new();
    /// assert_eq!(game.info().name, "zero-by");
    /// ```
    fn info() -> TargetData;
}

/* IMPLICIT GRAPH INTERFACE */

pub trait Transition<const B: usize = DEFAULT_STATE_BYTES> {
    /// Returns all possible abstract states that could proceed `state`.
    ///
    /// # Example
    ///
    /// In a discrete game, we represent points in history that have equivalent
    /// strategic value using a [`State<const B: usize>`] encoding. This is a
    /// bit-packed representation of the state of the game at a point in time
    /// (up to whatever attributes we may care about). This function returns the
    /// collection of all states that could follow `state` according to the
    /// underlying game's rules.
    ///
    /// Using the game [`zero_by`], whose default variant involves two players
    /// alternate turns removing items from a pile that starts out with 10 items
    /// (where Player 0 starts), we can provide the following example:
    ///
    /// ```
    /// use crate::game::zero_by;
    ///
    /// let mut game = zero_by::Session::new();
    /// let possible_next_states = vec![
    ///     "9-1".into(), // 9 items left, player 1's turn
    ///     "8-1".into(), // 8 items left, player 1's turn
    /// ];
    ///
    /// assert_eq!(game.prograde(game.start()), possible_next_states);
    /// ```
    ///
    /// # Warning
    ///
    /// In practice, it is extremely difficult to make it impossible for this
    /// function to always return an empty collection if `state` is invalid, as
    /// it is hard to statically verify the validity of a state. Hence, this
    /// behavior is only guaranteed when `state` is valid. See [`Bounded::end`]
    /// and [`Bounded::start`] to bound exploration to only valid states.
    fn prograde(&self, state: State<B>) -> Vec<State<B>>;

    /// Returns all possible abstract states that could preceed `state`.
    ///
    /// # Example
    ///
    /// In a discrete game, we represent points in history that have equivalent
    /// strategic value using a [`State<const B: usize>`] encoding. This is a
    /// bit-packed representation of the state of the game at a point in time
    /// (up to whatever attributes we may care about). This function returns the
    /// collection of all states that could have preceded `state` according to
    /// the underlying game's rules.
    ///
    /// Using the game [`zero_by`], whose default variant involves two players
    /// alternate turns removing items from a pile that starts out with 10 items
    /// (where Player 0 starts), we can provide the following example:
    ///
    /// ```
    /// use crate::game::zero_by;
    ///
    /// // Get state with 8 items left and player 1 to move
    /// let mut game = zero_by::Session::new();
    /// let state = game.decode("8-1".into())?;
    ///
    /// let possible_previous_states = vec![
    ///     "9-0".into(), // 9 items left, player 0's turn (invalid state)
    ///     "10-0".into(), // 8 items left, player 0's turn
    /// ];
    ///
    /// assert_eq!(game.retrograde(state), possible_previous_states);
    /// ```
    ///
    /// # Warning
    ///
    /// As you can see from the example, this function provides no guarantees
    /// about the validity of the states that it returns, because in the general
    /// case, it is impossible to verify whether or not a preceding state is
    /// actually valid.
    ///
    /// This obstacle is usually overcome by keeping track of observed states
    /// through a prograde exploration (using [`Transition::prograde`] and the
    /// functions provided by [`Bounded`]), and cross-referencing the outputs of
    /// this function with those observed states to validate them.
    fn retrograde(&self, state: State<B>) -> Vec<State<B>>;
}

pub trait Bounded<const B: usize = DEFAULT_STATE_BYTES> {
    /// Returns the starting state of the underlying target.
    ///
    /// Starting states are usually determined by target variants, but it is
    /// possible to alter them while remaining in the same game variant through
    /// the [`Forward`] interface. Such antics are necessary to ensure state
    /// validity at a variant-specific level. See [`Forward::forward`] for more.
    ///
    /// # Example
    ///
    /// Using the game [`zero_by`] with default state `"10-0"`:
    ///
    /// ```
    /// use crate::game::zero_by;
    /// let game = zero_by::Session::new();
    /// assert_eq!(game.encode(game.start())?, "10-0".into());
    /// ```
    fn start(&self) -> State<B>;

    /// Returns true if `state` is a terminal state of the underlying target.
    ///
    /// Note that this function could return `true` for an invalid `state`, so
    /// it is recommended that consumers verify that `state` is reachable in the
    /// first place through a traversal interface (see [`Transition`]).
    ///
    /// # Example
    ///
    /// Using the game [`zero_by`] as an example, which ends at any state with
    /// zero elements left:
    ///
    /// ```
    /// use crate::game::zero_by;
    /// let game = zero_by::Session::new();
    /// assert!(game.end(game.decode("0-0")?));
    /// ```
    fn end(&self, state: State<B>) -> bool;
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
    /// target variant.
    fn decode(&self, string: String) -> Result<State<B>>;

    /// Encodes a target `state` into a compact string representation.
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
    /// target variant.
    fn encode(&self, state: State<B>) -> Result<String>;
}

pub trait Variable {
    /// Initializes a version of the underlying target as the specified `variant`.
    ///
    /// A variant is a member of a family of targets whose structure is very
    /// similar. It is convenient to be able to express this because it saves
    /// a lot of needless re-writing of target logic, while allowing for a lot
    /// of generality in target implementations.
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
    /// Fails if `variant` does not conform to the target's protocol of encoding
    /// variants as strings, or if the target does not support variants in the
    /// first place (but has a placeholder [`Variable`] implementation).
    fn variant(variant: Variant) -> Result<Self>
    where
        Self: Sized;

    /// Returns a string representing the underlying target variant.
    ///
    /// This does not provide a certain way of differentiating between the
    /// starting state of the target (see [`Bounded::start`] for this), but it
    /// does provide a sufficient identifier of the target's structure.
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
    Self: Information + Bounded<B> + Codec<B> + Transition<B> + Sized,
{
    /// Sets the target's starting state to a pre-verified `state`.
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
        fallible and provides verification for target states."
    )]
    fn set_verified_start(&mut self, state: State<B>);

    /// Advances the target's starting state to the last state in `history`,
    /// verifying that it is a valid traversal of the induced graph on this
    /// target variant.
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

