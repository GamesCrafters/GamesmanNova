#![forbid(unsafe_code)]
//! # Game Module
//!
//! This module provides interfaces and implementations for sequential games.
//!
//! ## Working model
//!
//! The Nova project takes an unrestricted approach to the categories of games
//! it considers, but given its focus on efficient search, it is convenient to
//! specify some common assumptions and constructs created for ergonomics.
//!
//! In particular the following choices orient the project towards discrete
//! deterministic perfect-information sequential games:
//!
//! * [`State<const B: usize>`] is a bit-packed array of bytes used to identify
//!   game states (or in other words, equivalent game histories). This is backed
//!   by [`bitvec::array::BitArray`], which allows implementers to easily
//!   manipulate [`State`] instances.
//!
//! * The [`Transition`] interface encodes the rules of a discrete game by
//!   allowing implementers to specify which transitions between which states
//!   are legal according to the underlying ruleset.
//!
//! ## Provided traits
//!
//! The [`Bounded`] interface provides a way to begin and end a traversal. Such
//! a traversal can be carried out using the methods in [`Transition`]. Families
//! of games with common logic (e.g., the same board game played on bigger or
//! smaller boards) can be expressed as "variants" of each other through the
//! [`Variable`] interface.
//!
//! The [`GameData`] struct provides a structured way to communicate information
//! about a game, which is enabled by the [`Information`] trait. Furthermore, it
//! is possible to express native concepts like variants and states through
//! encodings specified by game implementations through the [`Codec`] interface
//! where necessary.
//!
//! For more complex tasks such as end-game analysis in large board games, it
//! can be desireable to artificially change the starting position of a game
//! without incurring the algorithmic cost of computation. The [`Forward`]
//! interface provides a verifiably correct way to do this.
//!
//! ## Implementing a new game
//!
//! The overarching hope is to make implementing a new game a matter of
//! selecting which structural interfaces it can satisfy, and of implementing
//! enough of the other interfaces to give it access to other functionality
//! (such as a solving algorithm in [`crate::solver::algorithm`]).
//!
//! Here are some concrete steps you can take to realize this:
//!
//! 1. **Determine the characteristics of your game:** Ascertain whether you are
//!    dealing with a chance game, a discrete game, a perfect-information game,
//!    etc. If you are dealing with anything that does not fit into the current
//!    working model, this is more of an infrastructure question, and you should
//!    reach out to a maintainer to talk about supporting a new game category.
//!
//! 2. **Set up a code skeleton:** Create a new submodule under this one, and
//!    give it the name of your game. Declare some kind of `Session` struct to
//!    represent the necessary information to encode an instance of your game.
//!    You should not need to mutate its state beyond initialization.
//!
//! 3. **Declare a set of interfaces:** Take a look at the provided traits, and
//!    declare the ones that seem to best fit the structure of your game and
//!    what you want to do with it. Reading documentation should help out a lot
//!    here.
//!
//! 4. **Reference existing implementations:** To actually implement the game,
//!    it will be very helpful to take a look at existing implementations. In
//!    particular, take a look at the [`zero_by`] module, which is an simple
//!    yet full-featured game implementation that we constantly make sure is
//!    up to standard.
//!
//! 5. **Write testing modules where appropriate:** If it happens that you have
//!    to implement anything that requires non-trivial logic, you should make
//!    sure to test it. This includes any kind of verification of encodings.
//!    Taking a look at existing unit tests will help significantly.

use anyhow::{Context, Result};

use crate::game::model::{State, Variant, DEFAULT_STATE_BYTES};

/* UTILITY MODULES */

#[cfg(test)]
mod test;
mod util;

pub mod model;
pub mod error;

/* MODULES */

#[cfg(test)]
pub mod mock;

pub mod crossteaser;
pub mod zero_by;

/* DEFINITIONS */

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

/// Provides a method to obtain information about a game.
pub trait Information {
    /// Provides a way to retrieve useful information about a game for both
    /// internal and user-facing modules.
    ///
    /// The information included here should be broadly applicable to any
    /// variant of the underlying game type (hence why it is a static method).
    /// For specifics on the information to provide, see [`GameData`].
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
    fn info() -> GameData;
}

/// Provides a method of bounding exploration of game states.
pub trait Bounded<const B: usize = DEFAULT_STATE_BYTES> {
    /// Returns the starting state of the underlying game.
    ///
    /// Starting states are usually determined by game variants, but it is
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

    /// Returns true if `state` is a terminal state of the underlying game.
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

/// Provides methods to encode and decode bit-packed [`State<B>`] instances to
/// and from [`String`]s to facilitate manual interfaces.
pub trait Codec<const B: usize = DEFAULT_STATE_BYTES> {
    /// Decodes a game `string` encoding into a bit-packed [`State<B>`].
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

/// Provides methods to obtain a working instance of a game variant and to
/// retrieve a [`String`]-encoded specification of the variant.
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
    /// Fails if `variant` does not conform to the game's protocol for encoding
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

/// Provides methods to safely fast-forward the starting state of a game to
/// a desired state in the future.
pub trait Forward<const B: usize = DEFAULT_STATE_BYTES>
where
    Self: Information + Bounded<B> + Codec<B> + Transition<B> + Sized,
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

    /// Advances the game's starting state to the last state in `history`.
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
    /// * The provided `history` plays beyond a terminal state.
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

/// Provides methods to obtain game state transitions, enabling state search.
pub trait Transition<const B: usize = DEFAULT_STATE_BYTES> {
    /// Returns all possible legal states that could follow `state`.
    ///
    /// In a discrete game, we represent points in history that have equivalent
    /// strategic value using a [`State<const B: usize>`] encoding. This is a
    /// bit-packed representation of the state of the game at a point in time
    /// (up to whatever attributes we may care about). This function returns the
    /// collection of all states that could follow `state` according to the
    /// underlying game's rules.
    ///
    /// # Example
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

    /// Returns all possible legal states that could have come before `state`.
    ///
    /// In a discrete game, we represent points in history that have equivalent
    /// strategic value using a [`State<const B: usize>`] encoding. This is a
    /// bit-packed representation of the state of the game at a point in time
    /// (up to whatever attributes we may care about). This function returns the
    /// collection of all states that could have preceded `state` according to
    /// the underlying game's rules.
    ///
    /// # Example
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
