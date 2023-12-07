//! # Game Implementations Module
//!
//! The `games` crate includes implementations for games intended to be
//! solved. To be able to solve a game with GamesmanNova, it must satisfy
//! the following characteristics/constraints:
//!
//! * It must have a finite amount of possible states and moves
//! * It must be reasonably "sized" (in terms of number of unique states)
//! * It must have states which can be efficiently represented
//! * It must not allow collaboration between players (WIP)
//!
//! This module includes functional constructs for economic games and for
//! methods that can be used to traverse them deterministically as well as
//! probabilistically. It also provides interfaces which make finding the
//! solution concepts for games we consider more efficient by taking advantage
//! of fundamental differences in their structure.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::{
    errors::NovaError,
    interfaces::terminal::cli::{IOMode, Solution},
    models::{Partition, PlayerCount, State, StateCount, Turn, Utility},
};
use nalgebra::{SMatrix, SVector};
use num_traits::Float;

/* IMPLEMENTED GAMES */

pub mod zero_by;

pub mod utils; // <==== Not a game

/* DATA CONSTRUCTS */

/// Contains useful data about a game, intended to provide users of the program
/// information they can use to understand the output of analysis and solving,
/// in addition to specifying game variants.
pub struct GameData {
    /* METADATA */
    /// Known name for the game. This should return a string that can be used as
    /// a command-line argument to the CLI endpoints which require a game name
    /// as a target (e.g. `nova solve <TARGET>`).
    pub name: String,
    /// The names of the people who implemented the game listed out, optionally
    /// including their contact. For example: "John Doe <john@rust-lang.org>,
    /// Ricardo L. <ricardo@go-lang.com>, Quin Bligh".
    pub authors: String,
    /// General introduction to the game's rules and setup, including any facts
    /// that are interesting about it.
    pub about: String,

    /* VARIANTS */
    /// Explanation of how to use strings to communicate which variant a user
    /// wishes to play to the game's implementation.
    pub variant_protocol: String,
    /// Regular expression pattern that all variant strings must match.
    pub variant_pattern: String,
    /// Default variant string to be used when none is specified.
    pub variant_default: String,

    /* STATES */
    /// Explanation of how to use a string to encode a game state.
    pub state_protocol: String,
    /// Regular expression pattern that all state encodings must match.
    pub state_pattern: String,
    /// Default state encoding to be used when none is specified.
    pub state_default: String,
}

/* ACCESS INTERFACES */

/// Defines miscellaneous behavior of a deterministic economic game object. Note
/// that player count is arbitrary; puzzles are semantically one-player games.
/// This interface is generic over **S**, the type encoding a state of the game.
///
/// ## Explanation
///
/// This interface is used to group together things that all such objects need
/// to do in relation to the rest of the project's modules, and not necessarily
/// in relation to their underlying nature. Namely, the methods in this
/// interface do not actually show the behavior of a game, only useful
/// information and procedures related to one for performing tasks which are
/// independent of the structure of the underlying game.
pub trait Game {
    /// Allows for the specification of a game variant and the initialization of
    /// a game's internal representation to a specific state. Calling this with
    /// a different `variant` argument should result in the `id` associated
    /// method returning a different string, meaning that game IDs should
    /// uniquely identify game variants. However, changing the initialization
    /// state `from` should make no difference in the output of `id`.
    ///
    /// It is important for the integrity of solution sets that implementers of
    /// this function verify that the provided `from` state is reachable from
    /// the starting state implied by the provided `variant`. As such, `variant`
    /// should be treated as the source of truth in the event that of the
    /// provided parameters is inconsistent with the other.
    ///
    /// Returns `Result::Ok(Self)` if the specified `variant` and `from` are
    /// not malformed. Otherwise, returns a `Result::Err(String)` containing a
    /// text string explaining why they could not be parsed.
    fn initialize(variant: Option<String>) -> Result<Self, NovaError>
    where
        Self: Sized;

    /* UTILITIES */

    /// Returns the verified variant string which was passed to the `initialize`
    /// function to obtain this game instance. Useful for providing rich errors
    /// with descriptive messages from generic code.
    fn variant(&self) -> String;

    /* SECONDARY PLUGINS
     *
     * The functions below facilitate secondary functionality used in features
     * of this project. Some of the interfaces in this file might help implement
     * the procedures they specify. If this is the case, a list of related
     * interfaces is listed in the docstring of each function.
     */

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
    /// - Calling `transition` on `history[i]` returns `history[i + 1]`.
    ///
    /// If these conditions are not satisfied, this function should return a
    /// useful error containing information about why the provided `history`
    /// is not possible for the game variant. Otherwise, it should mutate `self`
    /// to have a starting state whose string encoding is `history.pop()`, which
    /// can then be returned by whichever traversal interface is implemented by
    /// this game. Related interfaces:
    ///  
    /// - `Legible<S>`.
    fn forward(&mut self, history: Vec<String>) -> Result<(), NovaError>;

    /* MAIN PLUGINS
     *
     * These functions provide core functionality, and are directly associated
     * with an execution module (e.g., solving, analyzing, informing, etc.).
     * The interfaces declared in this file are structured around providing the
     * necessary functionality to implement these procedures. A list of related
     * interfaces is listed in the docstring of each function.
     */

    /// Returns useful information about the game, such as the type of game it
    /// is, who implemented it, and an explanation of how to specify different
    /// variants for initialization.
    fn info(&self) -> GameData;

    /// Runs a solving algorithm which consumes the callee, generating side
    /// effects specified by the `mode` parameter. This should return an error
    /// if solving the specific game variant is not supported (among other
    /// possibilities for an error), and a unit type if everything goes per
    /// specification. See `IOMode` for specifics on intended side effects.
    /// Related interfaces:
    ///
    /// - `Solvable<N>`.
    /// - Traversal interfaces (e.g., `StaticAutomaton<S, F>`).
    /// - Game structure markers (e.g., `Acyclic<N>`).
    fn solve(&self, mode: IOMode, method: Solution) -> Result<(), NovaError>;
}

/* MANUAL TRAVERSAL INTERFACES */

/// Defines the necessary behavior to encode and decode a state type **S** to
/// and from a `String`. This is related to the `GameData` object, which should
/// contain information about the way in which game states can be represented
/// using a string.
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
    Self: Game,
{
    /// Transforms a string representation of a game state into a type **S**.
    /// The `string` representation should conform to the `state_protocol`
    /// specified in the `GameData` object returned by `Game::info`. If it does
    /// not, an error containing a message with a brief explanation on what is
    /// wrong with `string` should be returned.
    fn decode(&self, string: String) -> Result<S, NovaError>;

    /// Transforms a game state type **S** into a string representation. The
    /// string returned should conform to the `state_protocol` specified in the
    /// `GameData` object returned by `Game::info`. If the `state` is malformed,
    /// this function should panic with a useful debug message. No two `state`s
    /// should return the same string representation (ideally).
    fn encode(&self, state: S) -> String;
}

/* DETERMINISTIC TRAVERSAL INTERFACES */

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
/// the behavior of games of differing player count. The notion of an NFA allows
/// for somewhat of a formal grappling with these similarities, as automata
/// theory has a lot of vocabulary to draw from. If you do not know what this
/// is, please see [the NFA wikipedia page](https://tinyurl.com/4vxdkvzh) for
/// more information.
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
/// the behavior of games of differing player count. The notion of an NFA allows
/// for somewhat of a formal grappling with these similarities, as automata
/// theory has a lot of vocabulary to draw from. If you do not know what this
/// is, please see [the NFA wikipedia page](https://tinyurl.com/4vxdkvzh) for
/// more information.
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

/* STOCHASTIC TRAVERSAL INTERFACES */

/// Defines the local behavior of a discrete markov model. Generic over **S**
/// (the type of the states within the model) and over **P** (the type used to
/// represent a probability primitive). An implementation of this trait allows
/// for an arbitrary number of transition states for any given state by using
/// heap-allocated variable-sized vectors in the `transition` function to return
/// an encoding of a probability mass function of arbitrary size.
///
/// ## Explanation
///
/// While solution concepts for nondeterministic games do not always provide
/// strategies that "guarantee" an outcome, there are still many interesting
/// statements to make about them. Most notably, it is possible to solve for
/// different notions of equilibrium within a probabilistic game, which also
/// requires a process of backwards induction, and hence benefits from this
/// computer program.
///
/// ## Note
///
/// While considering potentially infinite play is down to human decision in the
/// case of deterministic games, for many games there is a nonzero probability
/// associated with not having the game end after an arbitrary number of turns
/// **regardless** of player strategy.
pub trait DynamicMarkovModel<S, P>
where
    Self: Game,
    P: Float,
{
    /// Returns an encoding of the start state for an instance of this model.
    /// While this is could also model a fundamentally probabilistic process, it
    /// is left to the implementer to decide whether this function is pure.
    fn start(&self) -> S;

    /// Returns a vector of states `v` reachable from `state`, and another
    /// vector of the same size `p` of probabilities such that the probability
    /// of a transition from `state` to `v[i]` is `p[i]`. The sum of all values
    /// in `p` should be `1` (up to floating point accuracy).
    fn transition(&self, state: S) -> (Vec<S>, Vec<P>);
}

/// Defines the local behavior of a discrete markov model. Generic over **S**
/// (the type of the states within the model), **P** (the type used to represent
/// a probability primitive), and over **F** (the maximum state fan-out of the
/// transition function). A limitation on the number of transition states for
/// all states is imposed to gain the performance benefit of the usage of static
/// data structures in `transition`. See `DynamicMarkovModel` for an interface
/// that allows returning a PMF of arbitrary size from `transition`.
///
/// ## Explanation
///
/// While solution concepts for nondeterministic games do not always provide
/// strategies that "guarantee" an outcome, there are still many interesting
/// statements to make about them. Most notably, it is possible to solve for
/// different notions of equilibrium within a probabilistic game, which also
/// requires a process of backwards induction, and hence benefits from this
/// computer program.
///
/// ## Note
///
/// While considering potentially infinite play is down to human decision in the
/// case of deterministic games, for many games there is a nonzero probability
/// associated with not having the game end after an arbitrary number of turns
/// **regardless** of player strategy.
pub trait StaticMarkovModel<S, P, const F: usize>
where
    Self: Game,
    P: Float,
{
    /// Returns an encoding of the start state for an instance of this model.
    /// While this is could also model a fundamentally probabilistic process, it
    /// is left to the implementer to decide whether this function is pure.
    fn start(&self) -> S;

    /// Returns a static vector of states `v` reachable from `state` and another
    /// vector of the same size `p` of probabilities such that the probability
    /// of a transition from `state` to `v[i]` is `p[i]`. The sum of all values
    /// in `p` should be `1` (up to floating point accuracy).
    fn transition(&self, state: S) -> (SVector<S, F>, SVector<P, F>);
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
    fn utility(&self, state: State) -> SVector<Utility, N>;

    /// Returns the player `i` whose turn it is at the given `state`. The player
    /// identifier `i` should never be greater than `N - 1`, where `N` is the
    /// number of players in the underlying game.
    fn turn(&self, state: State) -> Turn;
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
    /// in a way that is equitable (improving efficiency).
    fn size(&self, partition: Partition) -> StateCount;
}

/* GAME STRUCTURE MARKERS */

/// Indicates that the graph induced by the underlying game's states is acyclic.
/// This intuitively means that no state will appear twice in a single session
/// of game play. It is very practical for a game to exhibit this structure, as
/// performing backwards induction is quite natural on acyclic graphs. Note that
/// there is no behavior associated with this trait; it is used as a marker for
/// providing blanket implementations from solvers which require games' state
/// graphs to not have any cycles.
pub trait Acyclic<const N: PlayerCount>: Solvable<N> {}

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
