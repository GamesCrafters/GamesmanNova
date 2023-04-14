//! # Archetypes Module
//!
//! This module contains overarching classes of games in the form of traits.
//! In general, these are not always mutually exclusive; it is the case that
//! they share many of the same fundamental characteristics. However, they are
//! separated in terms of the opportunity that is awarded by considering what
//! makes each one of them unique.
//!
//! This is mainly motivated by novel approaches to solving algorithms, but the
//! categories can be extended arbitrarily much. For example, different kinds
//! of interface to GamesmanNova might want to consider these categories
//! differently, or database implementations might be different when
//! considering the underlying structure of the game involved.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::{
    solvers::{AcyclicallySolvable, CyclicallySolvable, TierSolvable, TreeSolvable},
    State, Value, Variant,
};
use std::collections::HashSet;

/* BASE TRAITS */

/// A generic deterministic finite-state game or puzzle.
pub trait Game {
    /// Allows for the specification of a game variant and the initialization
    /// of a game's internal representation.
    fn initialize(variant: Option<Variant>) -> Self
    where
        Self: Sized;
    /// Returns the set of possible states one move away from `state`.
    fn adjacent(&self, state: State) -> HashSet<State>;
    /// Returns `None` if the state is non-terminal, and a `Value` otherwise.
    fn value(&self, state: State) -> Option<Value>;
    /// Returns the game's default state.
    fn default(&self) -> State;
    /// Returns an ID unique to this game and consistent across calls from the
    /// same variant.
    fn id(&self) -> String;
}

/* MARKABLE TRAITS */

/// One of the simplest types of game. Here, every ramification of the game is
/// mutually exclusive of all others -- if you choose to make a move from many,
/// there is no way of getting to a state as if you had made another.
pub trait TreeGame
where
    Self: Game,
    Self: AcyclicallySolvable,
    Self: CyclicallySolvable,
    Self: TierSolvable,
    Self: TreeSolvable,
{
}

/// In acyclic games, it is possible to get to a state in more than one way.
/// They are generally all
pub trait AcyclicGame
where
    Self: Game,
    Self: AcyclicallySolvable,
    Self: CyclicallySolvable,
    Self: TierSolvable,
{
}

/// In a tiered game, you can choose a way to split up the game state graph
/// into connected components such that they themselves form an acyclic graph,
/// which has significant implications for solving algorithms.
pub trait TieredGame
where
    Self: Game,
    Self: CyclicallySolvable,
    Self: TierSolvable,
{
}

/// In cyclic games, there are no guarantees as to whether or not you can
/// partition the state graph into tiers, reach each state uniquely, or have
/// all possible move sequences be finite.
pub trait CyclicGame
where
    Self: Game,
    Self: CyclicallySolvable,
{
}

/// A relatively small game.
pub trait SmallGame
where
    Self: Game,
{
}

/// A relatively large game.
pub trait LargeGame
where
    Self: Game,
{
}
