//! # Solvers Module
//!
//! This module provides behavior for the systematic traversal of game graphs
//! via their implementation of different interfaces defining their behavior,
//! with the objective of computing their solutions.

use anyhow::Result;
use rusqlite::Statement;
use rusqlite::Transaction;

use crate::game::DEFAULT_STATE_BYTES as DBYTES;
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::State;
use crate::interface::IOMode;

/* UTILITY MODULES */

pub mod util;
pub mod error;

/* MODULES */

pub mod db;
pub mod algorithm {
    pub mod acyclic;
}

/* TYPES */

/// Indicates the number of choices that players have to make to reach a
/// terminal state in a game under perfect play. For drawing positions,
/// indicates the number of choices players can make to bring the game to a
/// state which can transition to a non-drawing state.
pub type Remoteness = u32;

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

/// Values that solving algorithms calculate for each state within a game.
#[derive(Debug)]
pub struct Solution<const N: PlayerCount> {
    pub remoteness: Remoteness,
    pub utility: [IUtility; N],
    pub player: Player,
}

/// SQL query strings to be prepared into pre-compiled statements.
pub struct Queries {
    pub insert: String,
    pub select: String,
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
}

/* UTILITY MEASURE INTERFACES */

pub trait IntegerUtility<const N: PlayerCount, const B: usize = DBYTES>
where
    Self: Game<N, B>,
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
    Self: Game<N, B>,
{
    /// Returns the utility vector associated with a terminal `state` where
    /// whose `i`'th entry is the utility of the state for player `i`.
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
    fn utility(&self, state: State<B>) -> [SUtility; N];
}

/* UTILITY STRUCTURE INTERFACES */

pub trait ClassicGame<const B: usize = DBYTES>
where
    Self: Game<2, B>,
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
    fn utility(&self, state: State<B>) -> SUtility;
}

pub trait ClassicPuzzle<const B: usize = DBYTES>
where
    Self: Game<1, B>,
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

/* PERSISTENCE INTERFACES */

#[allow(async_fn_in_trait)]
pub trait Persistent<const N: PlayerCount, const B: usize = DBYTES> {
    /// Stores `info` under the key `state`, replacing an existing entry.
    ///
    /// This is used for persistence purposes. More information than `info` may
    /// be stored alongside `info` as a side effect. The effects of this may not
    /// persist unless `commit` is called afterwards.
    ///
    /// # Errors
    ///
    /// When `prepare` is not called before `insert`.
    fn insert(
        &mut self,
        stmt: &mut Statement,
        state: &State<B>,
        info: &Solution<N>,
    ) -> Result<()>;

    /// Retrieves the entry associated with `state`, or `None`.
    ///
    /// Entries are inserted through `insert`. The effects of this may not be
    /// persistent unless `commit` is called afterwards.
    ///
    /// # Errors
    ///
    /// When `prepare` is not called before `select`.
    fn select(
        &mut self,
        stmt: &mut Statement,
        state: &State<B>,
    ) -> Result<Option<Solution<N>>>;

    /// Prepares the underlying store for a series of calls to `insert` and
    /// `select`, according to `mode`.
    ///
    /// # Errors
    ///
    /// On a variety of conditions which depend on the underlying store.
    fn prepare(
        &mut self,
        tx: &mut Transaction,
        mode: IOMode,
    ) -> Result<Queries>;
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
