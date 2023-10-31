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

use crate::models::{Solver, State, Variant};
use nalgebra::{Matrix1, SMatrix, SVector, Vector1};
use std::collections::HashMap;

/* INTEGRATION */

pub mod zero_by;

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
pub trait Game
where
    Self: Automaton<State>,
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
/// that can solve the underlying game. Generic over **N** representing the
/// number of players that the game can be solved for.
///
/// Note that all solvable games must be traversable, which is why all
/// implementers of this trait must also conform to the `Automaton<State>`
/// interface with a state encoding of `State = u64`.
///
/// The method descriptions make ample mention of "utility." This notion is
/// purely abstract, and it stands for the "goodness" that a player associates
/// with a given state (much like the notion of "cost" is associated with
/// "badness"). By being able to determine the utility value each player gains
/// from all terminal positions in a game, we can fix which choice(s) they would
/// make when they have the power to transform the state of the game. In turn,
/// this allows us to explore the game tree and find strategies for the players
/// which maximize their net utility at the end of the game.
///
/// The general working framework assumes the following algorithm of which
/// state a player chooses to transition the game to on their turn, given that
/// the state is non-terminal:
///
/// ```none
/// Tuple result = (None, -inf)
/// Matrix W = game.weights(current_state)
/// Vector c = game.coalesce(current_state)
/// for each state s in game.transition(current_state):
///     Vector u = W.map(game.utility(s))
///     Scalar r = u.dot(c)
///     if r > result.1:
///         result = (s, r)
/// return result.0
/// ```
///
/// This roughly translates to "the player whose turn it is makes the move which
/// maximizes the utility of the players that they like, and minimizes the
/// utility for the players that they don't like, taking into consideration any
/// alliances they might have made."
pub trait Solvable<const N: usize>
where
    Self: Game,
{
    /// Returns a square matrix _W_ of dimension `N x N` (where `N` is the
    /// number of players in the game) such that the entry `W[i][j]` indicates
    /// the utility player `i` obtains for the utility of player `j`.
    ///
    /// For example, a _W_ with `N = 3`,
    ///
    /// ```none
    ///                     [[ 0,  7,  5],
    ///                 W =  [ 3, -1, -5],
    ///                      [-2,  1,  3]]
    /// ```
    ///
    /// ...indicates that Player 2 gains 3 utility units for each unit of
    /// utility Player 1 receives, because `W[2] == [3, 1, 5]`, and `W[2][1]
    /// == 3`.
    fn weights(&self) -> SMatrix<f64, N, N>;

    /// If `state` is terminal, returns the utility vector associated with that
    /// state, where `utility[i]` is the utility of the state for player `i`. If
    /// the state is not terminal, returns `None`, as non-terminal states
    /// represent no intrinsic utility to players.
    fn utility(&self, state: State) -> Option<SVector<f64, N>>;

    /// Given a `state`, returns an embedding C for the player(s) whose "turn it
    /// is." This idea is fairly abstract, so to exemplify, consider the initial
    /// state of a game of Tic-Tac-Toe. Since Player 0 always moves first and
    /// there are two players in total, this function would return the
    /// following vector:
    ///
    /// ```none
    ///                     C = [1, 0]
    /// ```
    ///
    /// Since this was returned on Player 0's "turn to move," this tells us that
    /// Player 0 wants to optimize for their own utility, without any regard
    /// for Player 1's utility. Now imagine a 6-player game of Chinese
    /// Checkers, where Player 0 and Player 1 are in a coalition. That is,
    /// they are acting non-selfishly. If it is Player 0's turn, this
    /// function could return the following vector:
    ///
    /// ```none
    ///                     C = [4, 1, 0, 0, 0, 0]
    /// ```
    ///
    /// This tells us that Player 0 provides some value to Player 1's utility,
    /// even if it has no actual utility for Player 0 (the actual utility Player
    /// 0 attributes to Player 1's utility is accounted for in `weights`).
    ///
    /// Additionally, note that if a player wants _everyone_ to lose "just as
    /// bad", this would be equivalent to wanting everyone to win "just as
    /// much". This functionally means that `kC = C`, which holds for all
    /// `k` integer values.
    fn coalesce(&self, state: State) -> SVector<f64, N>;

    /// Returns a mapping of names to solvers that can consume the implementer.
    /// That is, this function returns a named set of functions that can solve
    /// the game which returned them.
    fn solvers(&self) -> HashMap<&str, Solver<Self>>;
}

/* PUZZLE GAME BLANKET */

pub trait Puzzle
where
    Self: Solvable<1>,
{
}

impl<P> Solvable<1> for P
where
    P: Puzzle,
{
    fn weights(&self) -> SMatrix<f64, 1, 1>
    {
        Matrix1::new(1)
    }

    fn utility(&self, state: State) -> Option<SVector<f64, 1>>
    {
        if !self.accepts(state) {
            None
        } else {
            Some(Vector1::new(1))
        }
    }

    fn coalesce(&self, state: State) -> SVector<f64, 1>
    {
        Vector1::new(1)
    }
}

/* SOLVING MARKERS */

/// Indicates that a game's state graph can be partitioned into independent
/// connected components and solved taking advantage of this.
pub trait TierSolvable<const N: usize>
where
    Self: Solvable<N>,
{
}

/// Indicates that a game is solvable in a generally inefficient manner.
pub trait CyclicallySolvable<const N: usize>
where
    Self: Solvable<N>,
{
}

/// Indicates that a game is solvable using methods only available to games
/// whose state graphs are acyclic (which includes tree games).
pub trait AcyclicallySolvable<const N: usize>
where
    Self: Solvable<N>,
{
}

/// Indicates that a game is solvable using methods only available to games
/// with unique move paths to all states.
pub trait TreeSolvable<const N: usize>
where
    Self: Solvable<N>,
{
}
