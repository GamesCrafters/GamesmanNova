//! # Solvers Module
//!
//! This module provides behavior for the systematic traversal of game trees
//! via their implementation of different interfaces defining deterministic or
//! probabilistic behavior, with the objective of computing their strong or weak
//! solutions, or finding "solutions" under different game-theoretic definitions
//! of that word.
//!
//! #### Authorship
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)
//! - Ishir Garg, 4/3/2024 (ishirgarg@berkeley.edu)

use crate::model::{
    game::{
        Partition, Player, PlayerCount, State, StateCount,
        DEFAULT_STATE_BYTES as DBYTES,
    },
    solver::{IUtility, RUtility, SUtility},
};

/* MODULES */

/// Implementations of records that can be used by solving algorithms to store
/// or persist the information they compute about a game, and communicate it to
/// a database system.
pub mod record {
    pub mod mur;
    pub mod sur;
    pub mod rem;
}

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

#[cfg(test)]
mod test;
mod error;
mod util;

/* SOLVER DATABASE RECORDS */

/// A record layout that can be used to encode and decode the attributes stored
/// in serialized records. This is stored in database table schemas so that it
/// can be retrieved later for deserialization.
#[derive(Clone, Copy)]
pub enum RecordType {
    /// Multi-Utility Remoteness record for a specific number of players.
    MUR(PlayerCount),
    /// Simple Utility Remoteness record for a specific number of players.
    SUR(PlayerCount),
    /// Remoteness record (no utilities).
    REM,
}

/* STRUCTURAL INTERFACES */

/// TODO
pub trait Extensive<const N: PlayerCount, const B: usize = DBYTES> {
    /// Returns the player `i` whose turn it is at the given `state`. The player
    /// identifier `i` should never be greater than `N - 1`, where `N` is the
    /// number of players in the underlying game.
    fn turn(&self, state: State<B>) -> Player;

    /// Returns the number of players in the underlying game. This should be at
    /// least one higher than the maximum value returned by `turn`.
    #[inline(always)]
    fn players(&self) -> PlayerCount {
        N
    }
}

/// TODO
pub trait Composite<const N: PlayerCount, const B: usize = DBYTES>
where
    Self: Extensive<N, B>,
{
    /// Returns a unique identifier for the partition that `state` is an element
    /// of within the game variant specified by `self`. This implies no notion
    /// of ordering between identifiers.
    fn partition(&self, state: State<B>) -> Partition;

    /// Provides an arbitrarily precise notion of the number of states that are
    /// elements of `partition`. This can be used to distribute the work of
    /// analyzing different partitions concurrently across different consumers
    /// in a way that is equitable to improve efficiency.
    fn size(&self, partition: Partition) -> StateCount;
}

/* UTILITY MEASURE INTERFACES */

/// TODO
pub trait RealUtility<const N: PlayerCount, const B: usize = DBYTES> {
    /// If `state` is terminal, returns the utility vector associated with that
    /// state, where `utility[i]` is the utility of the state for player `i`. If
    /// the state is not terminal, it is recommended that this function panics.
    fn utility(&self, state: State<B>) -> [RUtility; N];
}

/// TODO
pub trait IntegerUtility<const N: PlayerCount, const B: usize = DBYTES> {
    /// If `state` is terminal, returns the utility vector associated with that
    /// state, where `utility[i]` is the utility of the state for player `i`. If
    /// the state is not terminal it is recommended that this function panics.
    fn utility(&self, state: State<B>) -> [IUtility; N];
}

/// TODO
pub trait SimpleUtility<const N: PlayerCount, const B: usize = DBYTES> {
    /// If `state` is terminal, returns the utility vector associated with that
    /// state, where `utility[i]` is the utility of the state for player `i`. If
    /// the state is not terminal, it is recommended that this function panics.
    fn utility(&self, state: State<B>) -> [SUtility; N];
}

/* UTILITY STRUCTURE INTERFACES */

/// Indicates that a game is 2-player, simple-sum, and zero-sum; this restricts
/// the possible utilities for a position to the following cases:
/// * `[Draw, Draw]`
/// * `[Lose, Win]`
/// * `[Win, Lose]`
/// * `[Tie, Tie]`
///
/// Since either entry determines the other, knowing one of the entries and the
/// turn information for a given state provides enough information to determine
/// both players' utilities.
pub trait ClassicGame<const B: usize = DBYTES> {
    /// If `state` is terminal, returns the utility of the player whose turn it
    /// is at that state. If the state is not terminal, it is recommended that
    /// this function panics.
    fn utility(&self, state: State<B>) -> SUtility;
}

/// Indicates that a game is a puzzle with simple outcomes. This implies that it
/// is 1-player and the only possible utilities obtainable for the player are:
/// * `Lose`
/// * `Draw`
/// * `Tie`
/// * `Win`
///
/// A winning state is usually one where there exists a sequence of moves that
/// will lead to the puzzle being fully solved. A losing state is one where any
/// sequence of moves will always take the player to either another losing state
/// or a state with no further moves available (with the puzzle still unsolved).
/// A draw state is one where there is no way to reach a winning state but it is
/// possible to play forever without reaching a losing state. A tie state is any
/// state that does not subjectively fit into any of the above categories.
pub trait ClassicPuzzle<const B: usize = DBYTES> {
    /// If `state` is terminal, returns the utility of the puzzle's player. If
    /// the state is not terminal, it is recommended that this function panics.
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
    G: Extensive<2, B> + ClassicGame<B>,
{
    fn utility(&self, state: State<B>) -> [SUtility; 2] {
        let mut sutility = [SUtility::TIE; 2];
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
