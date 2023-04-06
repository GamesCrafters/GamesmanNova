//! # Games Module
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use std::collections::HashSet;
use crate::{State, Move, Value};

/* TRAITS */

/// A generic deterministic finite-state game or puzzle.
pub trait Game {
    /// Returns the state of the game from which to base a solve.
    fn state(&self) -> State;
    /// Returns the state of the game after performing `mv` move on `state`.
    fn play(&self, state: State, mv: Move) -> State;
    /// Returns a set of possible moves that can be made from `state`.
    fn generate_moves(&self, state: State) -> HashSet<Move>;
    /// Returns `None` if the state is non-terminal, and a `Value` otherwise.
    fn value(&self, state: State) -> Option<Value>;
}

/// One of the simplest types of game. Here, every ramification of the game is
/// mutually exclusive of all others -- if you choose to make a move from many,
/// there is no way of getting to a state as if you had made another.
pub trait TreeGame {

}

/// In acyclic games, it is possible to get to a state in more than one way.
/// They are generally all 
pub trait AcyclicGame {

}

/// In a tiered game, you can choose a way to split up the game state graph
/// into connected components such that they themselves form an acyclic graph,
/// which has significant implications for solving algorithms.
pub trait TieredGame {

}

/// In cyclic games, there are no guarantees as to whether or not you can
/// partition the state graph into tiers, reach each state uniquely, or have
/// all possible move sequences be finite.
pub trait CyclicGame {

}
