//! # Solvers Module
//!
//! This module provides behavior for the systematic traversal of game trees
//! via their implementation of different interfaces defining deterministic or
//! probabilistic behavior, with the objective of computing their strong or weak
//! solutions, or finding "solutions" under different game-theoretic definitions
//! of that word.

use anyhow::Result;

use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::State;
use crate::game::DEFAULT_STATE_BYTES as DBYTES;

/* UTILITY MODULES */

mod util;
pub mod error;

/* MODULES */

/// Implementations of algorithms that can consume game implementations and
/// compute different features of interest associated with groups of states or
/// particular states.
pub mod algorithm {
    pub mod acyclic;
}

/* TYPES */

/// Indicates the "depth of draw" which a drawing position corresponds to.
/// For more information, see [this whitepaper](TODO). This value should be
/// 0 for non-drawing positions.
pub type DrawDepth = u64;

/// Indicates the number of choices that players have to make to reach a
/// terminal state in a game under perfect play. For drawing positions,
/// indicates the number of choices players can make to bring the game to a
/// state which can transition to a non-drawing state.
pub type Remoteness = u64;

/// Please refer to [this](https://en.wikipedia.org/wiki/Mex_(mathematics)).
pub type MinExclusion = u64;

/// A discrete measure of how "good" an outcome is for a given player.
/// Positive values indicate an overall gain from having played the game,
/// and negative values are net losses. The metric over abstract utility is
/// subjective.
pub type IUtility = i64;

/// A simple measure of hoe "good" an outcome is for a given player in a
/// game. The specific meaning of each variant can change based on the game
/// in consideration, but this is ultimately an intuitive notion.
#[derive(Clone, Copy)]
#[repr(i8)]
pub enum SUtility {
    Lose = -1,
    Tie = 0,
    Win = 1,
}

/* DEFINITIONS */

/// Denotes the quantization level of utility values under consideration.
#[derive(Copy, Clone)]
pub enum UtilityType {
    Integer,
    Simple,
    Real,
}

/* STRUCTURAL INTERFACES */

pub trait Game<const N: PlayerCount, const B: usize = DBYTES> {
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

/* UTILITY MEASURE INTERFACES */

pub trait IntegerUtility<const N: PlayerCount, const B: usize = DBYTES>
where 
    Self: Game<N, B>
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
    fn utility(&self, state: State<B>) -> [IUtility; N];
}

pub trait SimpleUtility<const N: PlayerCount, const B: usize = DBYTES>
where 
    Self: Game<N, B>
{
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

pub trait ClassicGame<const B: usize = DBYTES>
where 
    Self: Game<2, B>
{
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

pub trait ClassicPuzzle<const B: usize = DBYTES> 
where 
    Self: Game<1, B>
{
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

/* PERSISTENCE INTERFACES */

pub trait Persistent<S, const B: usize = DBYTES> {
    /// TODO
    fn persist(&self, state: &State<B>, info: &S) -> Result<()>;  

    /// TODO
    fn retrieve(&self, state: &State<B>) -> Result<Option<S>>;
}

/* BLANKET IMPLEMENTATIONS */

// All N-player simple-utility games are also N-player integer-utility games.
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

// All 2-player zero-sum games are also 2-player simple-utility games.
impl<const B: usize, G> SimpleUtility<2, B> for G
where
    G: ClassicGame<B>,
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

// All puzzles are also 1-player simple-utility games.
impl<const B: usize, G> SimpleUtility<1, B> for G
where
    G: ClassicPuzzle<B>,
{
    fn utility(&self, state: State<B>) -> [SUtility; 1] {
        [self.utility(state)]
    }
}

