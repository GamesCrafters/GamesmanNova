//! # Game Traits
//!
//! TODO

use anyhow::Context;
use anyhow::Result;

use crate::game::Component;
use crate::game::DEFAULT_STATE_BYTES;
use crate::game::GameData;
use crate::game::IUtility;
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::SUtility;
use crate::game::State;
use crate::game::Variant;

/* METADATA INTERFACES */

pub trait Information {
    /// Returns useful information about the game family. See [`GameData`].
    fn info() -> GameData;
}

/* REPRESENTATION INTERFACES */

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
    fn encode(&self, state: &State<B>) -> Result<String>;
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
    fn variant(variant: Option<Variant>) -> Result<Self>
    where
        Self: Sized;

    /// TODO
    fn name(&self) -> &str;
}

/* STRUCTURAL INTERFACES */

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
    fn outgoing(&self, state: &State<B>) -> Vec<State<B>>;

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
    fn sink(&self, state: &State<B>) -> bool;
}

pub trait Transpose<const B: usize = DEFAULT_STATE_BYTES> {
    /// TODO
    fn incoming(&self, state: &State<B>) -> Vec<State<B>>;
}

/* UTILILITY INTERFACES */

pub trait Advance<const B: usize = DEFAULT_STATE_BYTES>
where
    Self: Information + Codec<B> + Implicit<B> + Sized,
{
    /// Sets the game's starting state to a pre-verified `state`.
    ///
    /// This function is an auxiliary item for [`Advance::forward`]. While it
    /// needs to be implemented for [`Advance::forward`] to work, there should
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
        initial states should be done through [`Advance::forward`], which is \
        fallible and provides verification for game states."
    )]
    fn set_verified_start(&mut self, state: &State<B>);

    /// Advances the game's starting state to the last state in `history`,
    /// verifying that it is a valid traversal of the induced graph on this
    /// game variant.
    ///
    /// This can be useful for skipping a significant amount of computation in
    /// the process of performing subgame analysis. Requires an implementation
    /// of [`Advance::set_verified_start`] to ultimately change the starting
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
    fn advance(&mut self, history: Vec<String>) -> Result<()> {
        let to = crate::game::util::verify_state_history(self, history)
            .context("Specified invalid state history.")?;
        self.set_verified_start(&to);
        Ok(())
    }
}

pub trait Sequential<const N: PlayerCount, const B: usize = DEFAULT_STATE_BYTES>
{
    /// Returns the player `i` whose turn it is at the given `state`.
    ///
    /// In general, it can be assumed that the player whose turn it is at there
    /// starting state is Player 0, with the sole exception of games whose state
    /// has been forwarded.
    ///
    /// # Warning
    ///
    /// The player identifier `i` should never be greater than `N - 1`, where
    /// `N` is the number of players in the game. Violating this will definitely
    /// result in a program panic at some point. Unfortunately, there are not
    /// many good ways of enforcing this restriction at compilation time.
    fn turn(&self, state: &State<B>) -> Player;
}

pub trait Partition<const B: usize = DEFAULT_STATE_BYTES> {
    /// Returns the ID of the component that contains `state`.
    ///
    /// # Warning
    ///
    /// The component graph (with an edge component(a) -> component(b) for each
    /// pair (a, b) where b is in Sequential::transition(a)) should be acyclic.
    fn component(&self, state: &State<B>) -> Component;
}

/* UTILITY MEASURE */

pub trait IntegerUtility<
    const N: PlayerCount,
    const B: usize = DEFAULT_STATE_BYTES,
> where
    Self: Sequential<N, B>,
{
    /// Returns the utility vector associated with a terminal `state` where
    /// whose `i`'th entry is the utility of the state for player `i`.
    ///
    /// The behavior of this function is undefined in cases where `state` is not
    /// terminal. No assumptions are made about the possible utility values that
    /// players can obtain through playing the game, except that they can be
    /// represented with integers; the [`IUtility`] type serves this purpose.
    ///
    /// # Example
    ///
    /// An extreme example of such a game would be ten people fighting over
    /// each other's coins. Since the coins are discrete, it is only possible
    /// to gain utility in specific increments. We can model this hypothetical
    /// game through this interface.
    fn utility(&self, state: &State<B>) -> [IUtility; N];
}

pub trait SimpleUtility<
    const N: PlayerCount,
    const B: usize = DEFAULT_STATE_BYTES,
> where
    Self: Sequential<N, B>,
{
    /// Returns the utility vector associated with a terminal `state` where the
    /// `i`'th entry is the utility of the state for player `i`.
    ///
    /// The behavior of this function is undefined in cases where `state` is not
    /// terminal. This assumes that utility players' utility values can only
    /// be within the following categories:
    /// * [`SUtility::Lose`]
    /// * [`SUtility::Tie`]
    /// * [`SUtility::Win`]
    ///
    /// # Example
    ///
    /// In a 6-player game of Chinese Checkers, it is possible for one player
    /// to obtain a [`SUtility::Win`] by finishing first (in the event where
    /// utility is defined without 2nd through 6th places), and everyone else
    /// would be assigned a [`SUtility::Lose`].
    fn utility(&self, state: &State<B>) -> [SUtility; N];
}

/* UTILITY STRUCTURE */

pub trait ClassicGame<const B: usize = DEFAULT_STATE_BYTES>
where
    Self: Sequential<2, B>,
{
    /// Returns the utility of the only player whose turn it is at `state`.
    ///
    /// This assumes that `state` is terminal, that the underlying game is
    /// two-player and zero-sum. In other words, the only attainable pairs of
    /// utility values should be the following:
    /// * [`SUtility::Lose`] and [`SUtility::Win`]
    /// * [`SUtility::Win`] and [`SUtility::Lose`]
    /// * [`SUtility::Tie`] and [`SUtility::Tie`]
    ///
    /// # Example
    ///
    /// This game category is fairly intuitive in that most two-player board
    /// games fall into it. For example, in a game of Chess a [`SUtility::Win`]
    /// is recognized to be the taking of the opponent's king, where this also
    /// implies that the player who lost it is assigned [`SUtility::Lose`], and
    /// where any other ending is classified as a [`SUtility::Tie`].
    ///
    /// # Warning
    ///
    /// While the games that implement this interface should be zero-sum, the
    /// type system is not sufficiently rich to enforce such a constraint at
    /// compilation time, so sum specifications are generally left to semantics.
    fn utility(&self, state: &State<B>) -> SUtility;
}

pub trait ClassicPuzzle<const B: usize = DEFAULT_STATE_BYTES>
where
    Self: Sequential<1, B>,
{
    /// Returns the utility of the only player in the puzzle at `state`.
    ///
    /// This assumes that `state` is terminal. The utility structure implies
    /// that the game is 1-player, and that the only utility values attainable
    /// for the player are:
    /// * [`SUtility::Lose`]
    /// * [`SUtility::Tie`]
    /// * [`SUtility::Win`]
    ///
    /// # Example
    ///
    /// As a theoretical example, consider a Rubik's Cube. Here, we say that
    /// the solved state of the cube is a [`SUtility::Win`] for the player, as
    /// they have completed their objective.
    ///
    /// Now consider a crossword puzzle where you cannot erase words. It would
    /// be possible for the player to achieve a [`SUtility::Lose`] by filling
    /// out incorrect words and having no possible words left to write.
    ///
    /// Finally, a [`SUtility::Tie`] can be interpreted as reaching an outcome
    /// of the puzzle where it is impossible to back out of, but that presents
    /// no positive or negative impact on the player.
    fn utility(&self, state: State<B>) -> SUtility;
}
