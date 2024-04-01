//! # Game Implementations Module
//!
//! The `games` module includes implementations for games intended to be
//! solved. To be able to solve a game with GamesmanNova, it must satisfy
//! the following characteristics/constraints:
//!
//! * It must be reasonably "sized" (number of equivalent states)
//! * It must have states which can be efficiently represented
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use anyhow::Result;
use nalgebra::SMatrix;

use crate::{
    interface::{IOMode, SolutionMode},
    model::{Partition, PlayerCount, State, StateCount, Turn, Utility},
};

/* UTILITY MODULES */

#[cfg(test)]
mod test;
mod error;
mod util;

/* IMPLEMENTED GAMES */

pub mod crossteaser;
pub mod extensive;
pub mod zero_by;

/* DATA CONSTRUCTS */

/// Contains useful data about a game, intended to provide users of the program
/// information they can use to understand the output of solving algorithms,
/// in addition to specifying game variants.
pub struct GameData<'a> {
    /* INSTANCE */
    /// The variant string used to initialize the `Game` instance which returned
    /// this `GameData` object from its `info` associated method.
    pub variant: &'a String,

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

/* ACCESS INTERFACES */

/// Defines miscellaneous behavior of a deterministic economic game object. Note
/// that player count is arbitrary; puzzles are semantically one-player games,
/// although they are more alike to optimization problems than cases of games.
///
/// ## Explanation
///
/// This interface is used to group together things that all such objects need
/// to do in relation to the rest of the project's modules, and not necessarily
/// in relation to their underlying nature.
pub trait Game {
    /// Returns `Result::Ok(Self)` if the specified `variant` is not malformed.
    /// Otherwise, returns a `Result::Err(String)` containing a text string
    /// explaining why it could not be parsed.
    fn initialize(variant: Option<String>) -> Result<Self>
    where
        Self: Sized;

    /* SECONDARY PLUGINS */

    /// Returns an ID unique to this game. The return value should be consistent
    /// across calls from the same game and variant, but differing across calls
    /// from different games and variants. As such, it can be thought of as a
    /// string hash whose input is the game and variant (although it is not at
    /// all necessary that it conforms to any measure of hashing performance).
    fn id(&self) -> String;

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

    /* MAIN PLUGINS */

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

/* INTERFACING */

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
pub trait Legible<S>
where
    Self: Bounded<S>,
{
    /// Transforms a string representation of a game state into a type **S**.
    /// The `string` representation should conform to the `state_protocol`
    /// specified in the `GameData` object returned by `Game::info`. If it does
    /// not, an error containing a message with a brief explanation on what is
    /// wrong with `string` should be returned.
    fn decode(&self, string: String) -> Result<S>;

    /// Transforms a game state type **S** into a string representation. The
    /// string returned should conform to the `state_protocol` specified in the
    /// `GameData` object returned by `Game::info`. If the `state` is malformed,
    /// this function should panic with a useful debug message. No two `state`s
    /// should return the same string representation (ideally).
    fn encode(&self, state: S) -> String;
}

/* DETERMINISTIC TRAVERSAL INTERFACES */

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
pub trait Bounded<S>
where
    Self: Game,
{
    /// Returns the starting state of the underlying structure. This is used to
    /// deterministically initialize a traversal.
    fn start(&self) -> S;

    /// Returns true if and only if there are no possible transitions from the
    /// provided `state`. Inputting an invalid `state` is undefined behavior.
    fn end(&self, state: S) -> bool;
}

/// Defines the behavior that allows for traversing what could be best described
/// as a discrete automata (specifically, a NFA if at all) whose states are
/// encoded with type `S`. While such an automata only provisions a transition
/// function that provides state transitions in the prograde of time, here we
/// also provide a way to define a transition function which acts as if all
/// transitions defined by the automata were inverted (which provides a way to
/// express transitions in the retrograde of time).
///
/// As opposed to `STransition`, this interface trades off efficiency in the
/// interest of versatility by using dynamically-sized data structures to return
/// queries without explicitly upper-bounding the amount of states that each
/// transition query can return.
///
/// ### Explanation
///
/// Most extensive-form games this project pertains to have a well-defined way
/// of making state transitions in the prograde of time, namely, making moves
/// that transition game states. What may seem most confusing is doing this in
/// retrograde.
///
/// The reason this is desireable is to decrease the memory usage of many forms
/// of backwards induction. If it is necessary to have the information of the
/// states in `prograde(state)` to make a judgement on `state`, it is usually
/// necessary to "remember" the structure of the game during exploration in
/// order to figure out which states' information we can deduce from all of the
/// terminal states (whose information is independent from any other state).
///
/// However, if we have the ability to transition states in retrograde, we can
/// simply move in retrograde from terminal states, without having to search for
/// for the terminal states' pre-images through the entire game tree.
pub trait DTransition<S> {
    /// Given a `state` at time `t`, returns all states that are possible at
    /// time `t + 1`. This should only guarantee that if `state` is feasible and
    /// not an end state, then all returned states are also feasible; therefore,
    /// inputting an invalid or end `state` is undefined behavior. The order of
    /// the values returned is insignificant.
    fn prograde(&self, state: S) -> Vec<S>;

    /// Given a `state` at time `t`, returns all states that are possible at
    /// time `t - 1`. This should only guarantee that if `state` is feasible,
    /// then all returned states are also feasible; therefore, inputting an
    /// invalid `state` is undefined behavior. The order of the values returned
    /// is insignificant.
    fn retrograde(&self, state: S) -> Vec<S>;
}

/// Defines the behavior that allows for traversing what could be best described
/// as a discrete automata (specifically, a NFA if at all) whose states are
/// encoded with type `S`. While such an automata only provisions a transition
/// function that provides state transitions in the prograde of time, here we
/// also provide a way to define a transition function which acts as if all
/// transitions defined by the automata were inverted (which provides a way to
/// express transitions in the retrograde of time).
///
/// As opposed to `DTransition`, this interface trades off versatility in the
/// interest of efficiency by using statically-sized data structures to return
/// queries without the overhead of dynamic memory allocation.
///
/// ### Explanation
///
/// Most extensive-form games this project pertains to have a well-defined way
/// of making state transitions in the prograde of time, namely, making moves
/// that transition game states. What may seem most confusing is doing this in
/// retrograde.
///
/// The reason this is desireable is to decrease the memory usage of many forms
/// of backwards induction. If it is necessary to have the information of the
/// states in `prograde(state)` to make a judgement on `state`, it is usually
/// necessary to "remember" the structure of the game during exploration in
/// order to figure out which states' information we can deduce from all of the
/// terminal states (whose information is independent from any other state).
///
/// However, if we have the ability to transition states in retrograde, we can
/// simply move in retrograde from terminal states, without having to search for
/// for the terminal states' pre-images through the entire game tree.
pub trait STransition<S, const F: usize> {
    /// Given a `state` at time `t`, returns all states that are possible at
    /// time `t + 1`. This should only guarantee that if `state` is feasible and
    /// not an end state, then all returned states are also feasible; therefore,
    /// inputting an invalid or end `state` is undefined behavior. In the return
    /// value, `Some(S)` represents a valid state. The order of these values is
    /// insignificant.
    fn prograde(&self, state: S) -> [Option<S>; F];

    /// Given a `state` at time `t`, returns all states that are possible at
    /// time `t - 1`. This should only guarantee that if `state` is feasible,
    /// then all returned states are also feasible; therefore, inputting an
    /// invalid `state` is undefined behavior. In the return value, `Some(S)`
    /// represents a valid state. The order of these values is insignificant.
    fn retrograde(&self, state: S) -> [Option<S>; F];
}

/* SOLVING INTERFACES */

/// Indicates that an economic game object can have utility associated with
/// players at some of its states. Naturally, this means we can make statements
/// about their utility at other states in the game based on its structure. The
/// kind of statements we can make is not decided by the implementation of this
/// interface; it is decided by the nature of the underlying game (e.g., a
/// deterministic game might be able to be strongly solved if it is of complete
/// information, but we can always assign some utility to different players at
/// different game states regardless of this fact).
///
/// The semantics of the word "solvable" here just refer to being able to have
/// information about the utility of a game state from the utility of another,
/// due to the above reasons. The nature of the underlying game then decides the
/// specific kinds of "solving" that we can do.
pub trait Solvable<const N: PlayerCount>
where
    Self: Game,
{
    /// If `state` is terminal, returns the utility vector associated with that
    /// state, where `utility[i]` is the utility of the state for player `i`. If
    /// the state is not terminal, it is recommended that this function panics
    /// with a message indicating that an attempt was made to calculate the
    /// utility of a non-primitive state.
    fn utility(&self, state: State) -> [Utility; N];

    /// Returns the player `i` whose turn it is at the given `state`. The player
    /// identifier `i` should never be greater than `N - 1`, where `N` is the
    /// number of players in the underlying game.
    fn turn(&self, state: State) -> Turn;

    /// Returns the number of players in the underlying game. This should be at
    /// least one higher than the maximum value returned by `turn`.
    #[inline(always)]
    fn players(&self) -> PlayerCount {
        N
    }
}

/// Indicates that the directed graph _G_ induced by the structure of the
/// underlying game can be partitioned into partitions which themselves induce a
/// directed acyclic graph. Note that this does not necessarily mean that all
/// partitions will be strongly connected components of _G_.
///
/// This is useful because it allows the identification of which partitions of
/// states can be analyzed concurrently if the analysis of some states depends
/// on first analyzing all other "downstream" states, which is the case for all
/// forms of backwards induction.
///
/// The semantics of the word "composite" here come from the fact that you can
/// essentially "forget" about all traversed states once you transition into a
/// new partition, meaning that it is basically a new game at that point. Hence,
/// the original game can be interpreted as being composed of different games.
pub trait Composite<const N: PlayerCount>
where
    Self: Solvable<N>,
{
    /// Returns a unique identifier for the partition that `state` is an element
    /// of within the game variant specified by `self`. The notion of ordering
    /// across identifiers is left to the implementer, as it is dependent on how
    /// this function is used.
    fn partition(&self, state: State) -> Partition;

    /// Provides an arbitrarily precise notion of the number of states that are
    /// elements of `partition`. This can be used to distribute the work of
    /// analyzing different partitions concurrently across different consumers
    /// in a way that is equitable to improve efficiency.
    fn size(&self, partition: Partition) -> StateCount;
}

/* UTILITY INTERFACES */

/// Indicates that it is possible for players to gain utility from the utility
/// of other players. This is purely utilitarian, as this additional utility
/// would ideally be factored into the game outcomes via `Solvable::utility`.
/// This is useful in instances when a social analysis on specific situations
/// needs to be made without modifying existing logic.
pub trait External<const N: PlayerCount>
where
    Self: Solvable<N>,
{
    /// Returns an NxN matrix `M`, where the entry `M[i][j]` equals the utility
    /// obtained by player `i` for each unit of utility included in imputations
    /// of player `j`. This is somewhat akin to an economic externality.
    fn externality(&self) -> SMatrix<Utility, N, N>;
}
