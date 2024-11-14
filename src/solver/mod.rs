//! # Solvers Module
//!
//! This module provides behavior for the systematic traversal of game trees
//! via their implementation of different interfaces defining deterministic or
//! probabilistic behavior, with the objective of computing their strong or weak
//! solutions, or finding "solutions" under different game-theoretic definitions
//! of that word.

use crate::game::model::{
    Partition, Player, PlayerCount, State, StateCount,
    DEFAULT_STATE_BYTES as DBYTES,
};
use crate::solver::model::{IUtility, RUtility, SUtility};

/* UTILITY MODULES */

#[cfg(test)]
mod test;
mod util;

pub mod error;
pub mod model;

/* MODULES */

/// Implementations of algorithms that can consume game implementations and
/// compute different features of interest associated with groups of states or
/// particular states.
pub mod algorithm {
    /// Solving algorithms for games that are either of incomplete information
    /// or non-deterministic. The strategies used here diverge somewhat from the
    /// other solving procedures, as bringing in probability is a fundamental
    /// change.
    pub mod stochastic {
        pub mod acyclic;
        pub mod cyclic;
    }

    /// Solving algorithms for deterministic complete-information games that are
    /// able to generate complete solution sets (from which an equilibrium
    /// strategy can be distilled for any possible state in the game).
    pub mod strong {
        pub mod acyclic;
        pub mod cyclic;
    }

    /// Solving algorithms for deterministic complete-information games that
    /// only guarantee to provide an equilibrium strategy for the underlying
    /// game's starting position, but which do not necessarily explore entire
    /// game trees.
    pub mod weak {
        pub mod acyclic;
        pub mod cyclic;
    }
}

/* STRUCTURAL INTERFACES */

/// Indicates that a discrete game is played sequentially, which allows for
/// representing histories as discrete [`State`]s.
pub trait Sequential<const N: PlayerCount, const B: usize = DBYTES> {
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
    fn turn(&self, state: State<B>) -> Player;

    /// Returns the number of players in the underlying game.
    #[inline(always)]
    fn players(&self) -> PlayerCount {
        N
    }
}

/// Indicates that a game can be partitioned into sub-games that can be solved
/// somewhat independently of each other.
pub trait Composite<const N: PlayerCount, const B: usize = DBYTES> {
    /// Returns an identifier for a subset of the space of states which `state`
    /// belongs to.
    ///
    /// This method is generally useful for computing different sub-games in
    /// parallel, which is why it is desireable that partitions are independent
    /// of each other (meaning that we do not need to know information about one
    /// to generate information about the other).
    ///
    /// Additional properties are desirable. For example, we can try to create
    /// partitions of low "conductance," or in other words, which have a lot of
    /// states within partitions, but not many state transitions between them.
    /// Knowing how to do this effectively is hard, and it is an active area of
    /// research when talking about general graphs.
    ///
    /// # Example
    ///
    /// Consider a standard game of Tic-Tac-Toe. Here, the rules prevent players
    /// from removing any pieces placed on the board. This tells us that there
    /// are no transitions between states with the same number of pieces on the
    /// board. Hence, one way to implement this function would be to simply
    /// return the number of pieces left on the board. (This is an illustrative
    /// example; it would not provide very much of a computational benefit.)
    fn partition(&self, state: State<B>) -> Partition;

    /// Returns an estimate of the number of states in `partition`.
    ///
    /// It can be useful to know how many states are in a given partition for
    /// practical purposes, such as distributing the load of exploring different
    /// partitions in a balanced way; counting the states precisely would more
    /// often than not defeat the purpose of saving computation.
    ///
    /// For many purposes, it is also enough for the return value of this
    /// function to be accurate relative to other partitions, and not in actual
    /// magnitude. The precise semantics of the output are left unspecified.
    ///
    /// # Example
    ///
    /// An example heuristic that could be used to estimate the number of states
    /// in a game partition could be the amount of information required to be
    /// able to differentiate among states within it. For example, if there are
    /// more pieces on a board game partition than another, it is likely that it
    /// has more states in it.
    fn size(&self, partition: Partition) -> StateCount;
}

/* UTILITY MEASURE INTERFACES */

/// Indicates that a multiplayer game's players can only obtain utility values
/// that are real numbers.
pub trait RealUtility<const N: PlayerCount, const B: usize = DBYTES> {
    /// Returns the utility vector associated with a terminal `state` whose
    /// `i`'th entry is the utility of the state for player `i`.
    ///
    /// The behavior of this function is undefined in cases where `state` is not
    /// terminal. No assumptions are made about the possible utility values that
    /// players can obtain through playing the game, except that they can be
    /// represented with real numbers (see [`RUtility`]).
    ///
    /// # Example
    ///
    /// An extreme example of such a game would be a discrete auction, where
    /// players can gain arbitrary amounts of money, but can only act at known
    /// discrete points in time.
    fn utility(&self, state: State<B>) -> [RUtility; N];
}

/// Indicates that a multiplayer game's players can only obtain utility values
/// that are integers.
pub trait IntegerUtility<const N: PlayerCount, const B: usize = DBYTES> {
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
    fn utility(&self, state: State<B>) -> [IUtility; N];
}

/// Indicates that a multiplayer game's players can only obtain utility values
/// within specific known categories.
pub trait SimpleUtility<const N: PlayerCount, const B: usize = DBYTES> {
    /// Returns the utility vector associated with a terminal `state` where
    /// whose `i`'th entry is the utility of the state for player `i`.
    ///
    /// The behavior of this function is undefined in cases where `state` is not
    /// terminal. This assumes that utility players' utility values can only
    /// be within the following categories:
    /// * [`SUtility::Lose`]
    /// * [`SUtility::Draw`]
    /// * [`SUtility::Tie`]
    /// * [`SUtility::Win`]
    ///
    /// # Example
    ///
    /// In a 6-player game of Chinese Checkers, it is possible for one player
    /// to obtain a [`SUtility::Win`] by finishing first (in the event where
    /// utility is defined without 2nd through 6th places), and everyone else
    /// would be assigned a [`SUtility::Lose`].
    fn utility(&self, state: State<B>) -> [SUtility; N];
}

/* UTILITY STRUCTURE INTERFACES */

/// Indicates that a game is 2-player, has categorical utility values, and is
/// zero-sum, allowing the implementer to eschew providing a player's utility.
pub trait ClassicGame<const B: usize = DBYTES> {
    /// Returns the utility of the only player whose turn it is at `state`.
    ///
    /// This assumes that `state` is terminal, that the underlying game is
    /// two-player and zero-sum. In other words, the only attainable pairs of
    /// utility values should be the following:
    /// * [`SUtility::Draw`] and [`SUtility::Draw`]
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
    /// In other games where it is possible to play forever, both players can
    /// achieve a [`SUtility::Draw`] by constantly avoiding reaching a terminal
    /// state (perhaps motivated by avoiding realizing imminent losses).
    ///
    /// # Warning
    ///
    /// While the games that implement this interface should be zero-sum, the
    /// type system is not sufficiently rich to enforce such a constraint at
    /// compilation time, so sum specifications are generally left to semantics.
    fn utility(&self, state: State<B>) -> SUtility;
}

/// Provides a method to find the utility of terminal values in "classic"
/// puzzles, which are single-player games with categorical [`SUtility`] values.
pub trait ClassicPuzzle<const B: usize = DBYTES> {
    /// Returns the utility of the only player in the puzzle at `state`.
    ///
    /// This assumes that `state` is terminal. The utility structure implies
    /// that the game is 1-player, and that the only utility values attainable
    /// for the player are:
    /// * [`SUtility::Lose`]
    /// * [`SUtility::Draw`]
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
    /// For a [`SUtility::Draw`], we can consider a puzzle where it is possible
    /// for a loss to be the only material option, but for there to also be the
    /// option to continue playing forever (as to not realize this outcome).
    ///
    /// Finally, a [`SUtility::Tie`] can be interpreted as reaching an outcome
    /// of the puzzle where it is impossible to back out of, but that presents
    /// no positive or negative impact on the player.
    fn utility(&self, state: State<B>) -> SUtility;
}

/* BLANKET IMPLEMENTATIONS */

impl<const N: PlayerCount, const B: usize, G> RealUtility<N, B> for G
where
    G: IntegerUtility<N, B>,
{
    fn utility(&self, state: State<B>) -> [RUtility; N] {
        let iutility = self.utility(state);
        let mut rutility = [0.0; N];
        rutility
            .iter_mut()
            .enumerate()
            .for_each(|(i, u)| *u = iutility[i] as RUtility);
        rutility
    }
}

impl<const N: PlayerCount, const B: usize, G> IntegerUtility<N, B> for G
where
    G: SimpleUtility<N, B>,
{
    fn utility(&self, state: State<B>) -> [IUtility; N] {
        let sutility = self.utility(state);
        let mut iutility = [0; N];
        iutility
            .iter_mut()
            .enumerate()
            .for_each(|(i, u)| *u = sutility[i].into());
        iutility
    }
}

impl<const B: usize, G> SimpleUtility<2, B> for G
where
    G: Sequential<2, B> + ClassicGame<B>,
{
    fn utility(&self, state: State<B>) -> [SUtility; 2] {
        let mut sutility = [SUtility::Tie; 2];
        let utility = self.utility(state);
        let turn = self.turn(state);
        let them = (turn + 1) % 2;
        sutility[them] = !utility;
        sutility[turn] = utility;
        sutility
    }
}

impl<const B: usize, G> SimpleUtility<1, B> for G
where
    G: ClassicPuzzle<B>,
{
    fn utility(&self, state: State<B>) -> [SUtility; 1] {
        [self.utility(state)]
    }
}
