//! # Solvers Module
//!
//! This module provides behavior for the systematic traversal of game trees
//! via their implementation of different interfaces defining deterministic or
//! probabilistic behavior, with the objective of computing their strong or weak
//! solutions, or finding "solutions" under different game-theoretic DEFINITIONS
//! of that word.
//!
//! ## Development Notes
//!
//! The semantics of this project have been set up such that implementing
//! different sets of interfaces should grant game implementations plugin-like
//! access to different solving algorithms with little to no performance
//! overhead due to abstractions. In this module, you will find that solvers are
//! optimized for the interfaces they provide blanket implementations for, and
//! the module structure should reflect that.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

/* CONSTANTS */

/// Describes the maximum number of states that are one move away from any state
/// within a game. Used to allocate statically-sized arrays on the stack for
/// faster execution of solving algorithms. If this limit is violated by a game
/// implementation, this program should panic.
pub const MAX_TRANSITIONS: usize = 50;

/* SOLVER MODULES */

/// Solving algorithms for games that are either of incomplete information or
/// non-deterministic. The strategies used here diverge somewhat from the other
/// solving procedures, as bringing in probability is a fundamental change.
pub mod stochastic {
    pub mod acyclic;
    pub mod cyclic;
}

/// Solving algorithms for deterministic complete-information games that are
/// able to generate complete solution sets (from which an equilibrium strategy
/// can be distilled for any possible state in the game).
pub mod strong {
    pub mod acyclic;
    pub mod cyclic;
}

/// Solving algorithms for deterministic complete-information games that only
/// guarantee to provide an equilibrium strategy for the underlying game's
/// starting position, but which do not necessarily explore entire game trees.
pub mod weak {
    pub mod acyclic;
    pub mod cyclic;
}

/* UTILITY MODULES */

pub mod error;
pub mod util;
