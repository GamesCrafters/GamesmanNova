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

use crate::models::{Solver, State, Value, Variant};

/* INTEGRATION MACROS */

/* Looks in this directory (games/) and expands to a collection of
 * module definitions as follows:
 *
 * ```
 * pub mod game_1;
 * pub mod game_2;
 * ...
 * pub mod game_n;
 * ```
 */
dirmod::all!(default pub);

/* Does the same thing, but instead of generating module definitions it
 * automatically creates a constant list of their names as follows:
 *
 * ```
 * pub const LIST: [&str; n] = [
 *    "game_1",
 *    "game_2",
 *    ...
 *    "game_n",
 * ];
 * ```
 */
list_modules::here!("src/games/");

/* DATA CONSTRUCTS */

/// Contains useful data about a game, intended to provide users of the program
/// information they can use to understand the output of analysis and solving,
/// in addition to specifying game variants.
pub struct GameData
{
    /// Known name for the game.
    pub name: String,

    /// The people who implemented the game.
    pub author: String,

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

/* FUNCTIONAL CONSTRUCTS */

/// Defines miscellanous behavior of a deterministic economic game object. Note
/// that player count is not specified, so puzzles are interpreted as one-player
/// games.
///
/// This interface is used to group together things that all such objects need
/// to do in relation to the rest of the project's modules, and not necessarily
/// in relation to their underlying nature. Namely, the methods in this
/// interface do not actually show the behavior of a game, only useful
/// information and procedures related to one for performing tasks which are
/// independent of the structure of the underlying game.
pub trait Game
{
    /// Allows for the specification of a game variant and the initialization of
    /// a game's internal representation. Calling this with a different
    /// `variant` argument should result in the `id(&self) -> String` associated
    /// method returning a different string, meaning that game IDs should
    /// uniquely identify game variants.
    fn initialize(variant: Option<Variant>) -> Self
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
}

/// Defines the behavior of a nondeterministic finite automaton _M_. Generic
/// over **S**, the type encoding a member of the set of states of _M_.
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
/// transformations that can be applied to any satate of the puzzle is finite.
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
pub trait Automaton<S>
{
    /// Returns an encoding of the start state for any automatic run in this
    /// automaton. All sequences in the formal language accepted by this
    /// automaton must satisfy that this function returns their first
    /// element.
    fn start(&self) -> S;

    /// Returns a vector of states reachable from `state` in this automaton.
    /// Formally, this is a mapping _T : S -> P(S)_ such that an element _x_ of
    /// _S_ is mapped to the set of other elements of _S_ reachable from _x_
    /// by accepting an arbitrary element of the input alphabet, where _S_ is
    /// the set of all possible states that this automaton can be in, and
    /// _P(S)_ is the power set of _S_.
    fn transition(&self, state: S) -> Vec<S>;

    /// Returns true if `state` is an encoding of an element of the accepting
    /// (or "final") set of states of this automaton. All sequences in the
    /// formal language accepted by this automaton must satisfy that this
    /// function returns their last element.
    fn accepts(&self, state: S) -> bool;
}

/// Indicates that an economic game object can be consumed by at least one
/// solving algorithm, and offers an associated method to retrieve the solvers
/// that can solve the underlying game. Generic over a type **V** representing
/// the different possibilities of a primitive strategic value associated with a
/// game state (i.e. win, lose, or tie).
///
/// Note that all solvable games must be traversable, which is why all
/// implementers of this trait must also conform to the `Automaton<State>`
/// interface with a state encoding of `State = u64`.
pub trait Solvable
where
    Self: Game,
    Self: Automaton<State>,
{
    /// Returns `None` if the state is non-terminal, and a `Value` otherwise.
    fn value(&self, state: State) -> Option<Value>;

    /// Returns all the solvers available to solve the game in order of
    /// overall efficiency, including their interface names. The option
    /// to choose a default solver in the implementation of this function
    /// is allowed by making one of them mapped to `None`, as opposed to
    /// `Some(String)`.
    fn solvers(&self) -> Vec<(String, Solver<Self>)>;
}

/* SOLVING MARKERS */

/// Indicates that a game's state graph can be partitioned into independent
/// connected components and solved taking advantage of this.
pub trait TierSolvable
where
    Self: Solvable,
{
}

/// Indicates that a game is solvable in a generally inefficient manner.
pub trait CyclicallySolvable
where
    Self: Solvable,
{
}

/// Indicates that a game is solvable using methods only available to games
/// whose state graphs are acyclic (which includes tree games).
pub trait AcyclicallySolvable
where
    Self: Solvable,
{
}

/// Indicates that a game is solvable using methods only available to games
/// with unique move paths to all states.
pub trait TreeSolvable
where
    Self: Solvable,
{
}
