//! # Game Implementations Module
//!
//! The `games` crate includes implementations for games intended to be
//! solved. To be able to solve a game with GamesmanNova, it must satisfy
//! the following characteristics/constraints:
//!
//! * It must have a finite amount of possible states and moves
//! * No probability must be involved in state transitions
//! * It must be reasonably "sized" (in terms of number of unique states)
//!
//! This module includes functional constructs for economic games and for
//! automata that can be used to traverse them. Additionally, it provides
//! interfaces for solving different kinds of games, such as puzzles.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::{
    errors::NovaError,
    interfaces::terminal::cli::IOMode,
    models::{Partition, State, StateCount, Utility, Variant},
};
use nalgebra::SVector;

/* INTEGRATION */

pub mod utils;
pub mod crossteaser;
pub mod zero_by;

/* DATA CONSTRUCTS */

/// Contains useful data about a game, intended to provide users of the program
/// information they can use to understand the output of analysis and solving,
/// in addition to specifying game variants.
pub struct GameData {
    /// Known name for the game. This should return a string that can be used as
    /// a command-line argument to the CLI endpoints which require a game name
    /// as a target (e.g. `nova solve <TARGET>`).
    pub name: String,

    /// The names of the people who implemented the game listed out, optionally \
    /// including their contact. For example: "John Doe <john@rust-lang.org>,
    /// Ricardo L. <ricardo@go-lang.com>, Quin Bligh".
    pub authors: String,

    /// The category of economic game (puzzle, two-player, etc.).
    pub category: String,

    /// General introduction to the game.
    pub about: String,

    /// Explanation of how to use strings to communicate which variant a user
    /// wishes to play to the game's implementation.
    pub variant_protocol: String,

    /// Regular expression pattern that all variant strings must match.
    pub variant_pattern: String,

    /// Default variant string to be used when none is specified.
    pub variant_default: String,
}

/* ACCESS INTERFACE */

/// Defines miscellaneous behavior of a deterministic economic game object. Note
/// that player count is not specified, so puzzles are interpreted as one-player
/// games.
///
/// This interface is used to group together things that all such objects need
/// to do in relation to the rest of the project's modules, and not necessarily
/// in relation to their underlying nature. Namely, the methods in this
/// interface do not actually show the behavior of a game, only useful
/// information and procedures related to one for performing tasks which are
/// independent of the structure of the underlying game.
pub trait Game {
    /// Allows for the specification of a game variant and the initialization of
    /// a game's internal representation. Calling this with a different
    /// `variant` argument should result in the `id(&self) -> String` associated
    /// method returning a different string, meaning that game IDs should
    /// uniquely identify game variants.
    ///
    /// Returns `Result::Ok(Self)` if the specified `variant` is properly
    /// formed. Otherwise, returns a `Result::Err(String)` containing a text
    /// string explaining why the variant string could not be parsed.
    fn initialize(variant: Option<Variant>) -> Result<Self, NovaError>
    where
        Self: Sized;

    /// Returns an ID unique to this game. The return value should be consistent
    /// across calls from the same game and variant, but differing across calls
    /// from different games and variants. As such, it can be thought of as a
    /// string hash whose input is the game and variant (although it is not at
    /// all necessary that it conforms to any measure of hashing performance).
    fn id(&self) -> String;

    /// Returns useful information about the game, such as the type of game it
    /// is, who implemented it, and an explanation of how to specify different
    /// variants for initialization.
    fn info(&self) -> GameData;

    /// Runs a solving algorithm which consumes the callee, generating side
    /// effects specified by the `mode` parameter. This should return an error
    /// if solving the specific game variant is not supported (among other
    /// possibilities for an error), and a unit type if everything goes per
    /// specification.
    fn solve(&self, mode: Option<IOMode>) -> Result<(), NovaError>;
}

/* TRAVERSAL INTERFACES */

/// Defines the behavior of a nondeterministic finite automaton _M_. Generic
/// over **S**, the type encoding a member of the set of states of _M_. An
/// implementation of this trait allows for an arbitrary number of transition
/// states for any given state by using a heap-allocated variable-sized vector
/// in the `transition` function. See `StaticAutomaton` for a static equivalent
/// of this interface.
///
/// ## Explanation
///
/// The motivation behind introducing this is the fundamental overlap between
/// the behavior of puzzles and games. The notion of an NFA allows for a formal
/// grappling with these similarities, as automata theory is quite well
/// established. If you do not know what this is, please see
/// [the NFA wikipedia page](https://tinyurl.com/4vxdkvzh) for more information.
///
/// ### Game Automata
///
/// For deterministic finite games, the final (or "accepting") state set _F_ of
/// an automaton _M_ modeling its behavior would be all of the end states (e.g.,
/// a board of tic-tac-toe with no more empty slots). Then, the formal language
/// _L(M)_ recognized by this automaton would contain all valid sequences of
/// moves leading from the start state of the game to a final state in _F_, with
/// the understanding that a finite alphabet is formed by all possible moves in
/// the game.
///
/// ### Puzzle Automata
///
/// In the case of puzzle automata, the final state set _F_ is similarly the set
/// of states where the puzzle is considered solved, and the language recognized
/// by them is the set of all combinations of transformations that can be
/// applied to the puzzle which bring it from the start state to a member of
/// _F_. Again, this is with the understanding that the amount of
/// transformations that can be applied to any state of the puzzle is finite.
///
/// ## Example
///
/// Consider a puzzle P which consists of removing elements from a set E one at
/// a time until there are no more elements. Then, we can define _M_ an NFA to
/// model P in the following manner:
///
/// - **S** `:= Unsigned Integer`, as sets can only have sizes in [0, _inf_).
/// - The set of states of _M_ is `{|E|, |E| - 1, ..., 0}`, which are all
///   intermediary sizes of E during the existence of the puzzle.
/// - The set of final (or accepting) states of _M_ is `{0}`, which contains the
///   size of the empty set.
/// - _M_'s recognized language _L(M)_ is the set containing only the sequence
///   `{1, 1, 1, ..., 1}` of length _|E|_, as you can only remove one element at
///   any given state, and there are _|E|_ states.
///
/// Note that _|L(M)|_ = 1 implies that there is only one way to solve this
/// puzzle. If _|L(M)|_ were equal to 0, that would imply the puzzle cannot be
/// solved.
pub trait DynamicAutomaton<S>
where
    Self: Game,
{
    /// Returns an encoding of the start state for any automatic run in this
    /// automaton. All sequences in the formal language accepted by this
    /// automaton must satisfy that this function returns their first
    /// element.
    fn start(&self) -> S;

    /// Returns a vector of states reachable from `state` in this automaton.
    /// This is a mapping _T : S -> P(S)_ such that an element _x_ of _S_ is
    /// mapped to the set of other elements of _S_ reachable from _x_ by
    /// accepting an arbitrary element of the input alphabet, where _S_ is
    /// the set of all possible states that this automaton can be in, and
    /// _P(S)_ is the power set of _S_. If the return value of this function is
    /// empty, we assume that the automata accepts `state`.
    fn transition(&self, state: S) -> Vec<S>;
}

/// Defines the behavior of a nondeterministic finite automaton _M_. Generic
/// over **S** (the type encoding a member of the set of states of _M_) and
/// **F** (the maximum fan-out of the transition function). A limitation on
/// the number of transition states for all states is imposed to gain the
/// performance benefit of the usage of static arrays in `transition`. See
/// `DynamicAutomaton` for an interface that allows returning an arbitrary
/// number of states in `transition`.
///
/// ## Explanation
///
/// The motivation behind introducing this is the fundamental overlap between
/// the behavior of puzzles and games. The notion of an NFA allows for a formal
/// grappling with these similarities, as automata theory is quite well
/// established. If you do not know what this is, please see
/// [the NFA wikipedia page](https://tinyurl.com/4vxdkvzh) for more information.
///
/// ### Game Automata
///
/// For deterministic finite games, the final (or "accepting") state set _F_ of
/// an automaton _M_ modeling its behavior would be all of the end states (e.g.,
/// a board of tic-tac-toe with no more empty slots). Then, the formal language
/// _L(M)_ recognized by this automaton would contain all valid sequences of
/// moves leading from the start state of the game to a final state in _F_, with
/// the understanding that a finite alphabet is formed by all possible moves in
/// the game.
///
/// ### Puzzle Automata
///
/// In the case of puzzle automata, the final state set _F_ is similarly the set
/// of states where the puzzle is considered solved, and the language recognized
/// by them is the set of all combinations of transformations that can be
/// applied to the puzzle which bring it from the start state to a member of
/// _F_. Again, this is with the understanding that the amount of
/// transformations that can be applied to any state of the puzzle is finite.
///
/// ## Example
///
/// Consider a puzzle P which consists of removing elements from a set E one at
/// a time until there are no more elements. Then, we can define _M_ an NFA to
/// model P in the following manner:
///
/// - **S** `:= Unsigned Integer`, as sets can only have sizes in [0, _inf_).
/// - The set of states of _M_ is `{|E|, |E| - 1, ..., 0}`, which are all
///   intermediary sizes of E during the existence of the puzzle.
/// - The set of final (or accepting) states of _M_ is `{0}`, which contains the
///   size of the empty set.
/// - _M_'s recognized language _L(M)_ is the set containing only the sequence
///   `{1, 1, 1, ..., 1}` of length _|E|_, as you can only remove one element at
///   any given state, and there are _|E|_ states.
///
/// Note that _|L(M)|_ = 1 implies that there is only one way to solve this
/// puzzle. If _|L(M)|_ were equal to 0, that would imply the puzzle cannot be
/// solved.
pub trait StaticAutomaton<S, const F: usize>
where
    Self: Game,
{
    /// Returns an encoding of the start state for any automatic run in this
    /// automaton. All sequences in the formal language accepted by this
    /// automaton must satisfy that this function returns their first
    /// element.
    fn start(&self) -> S;

    /// Returns a vector of states reachable from `state` in this automaton.
    /// This is a mapping _T : S -> P(S)_ such that an element _x_ of _S_ is
    /// mapped to the set of other elements of _S_ reachable from _x_ by
    /// accepting an arbitrary element of the input alphabet, where _S_ is
    /// the set of all possible states that this automaton can be in, and
    /// _P(S)_ is the power set of _S_. If the return value of this function
    /// contains only `Option::None` variants, we assume that the automata
    /// accepts `state`.
    fn transition(&self, state: S) -> [Option<S>; F];
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
pub trait Solvable<const N: usize>
where
    Self: Game,
{
    /// If `state` is terminal, returns the utility vector associated with that
    /// state, where `utility[i]` is the utility of the state for player `i`. If
    /// the state is not terminal, it is recommended that this function panics
    /// with a message indicating that an attempt was made to calculate the
    /// utility of a non-primitive state.
    fn utility(&self, state: State) -> SVector<Utility, N>;
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
pub trait Composite<const N: usize>
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
    /// in a way that is equitable (improving efficiency).
    fn size(&self, partition: Partition) -> StateCount;
}
