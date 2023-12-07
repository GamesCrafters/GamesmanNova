//! # Solvers Module
//!
//! This module provides behavior for the systematic traversal of game trees
//! via their implementation of different interfaces defining deterministic or
//! probabilistic behavior, with the objective of computing their strong or weak
//! solutions, or finding different equilibria in nondeterministic cases.
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
//! To make the possible combinations of characteristics of games that we
//! consider clearer, each game can be:
//!
//! - Deterministic or probabilistic
//! - Cyclic or acyclic (game states can repeat or not)
//! - Of any positive player count
//!
//! To make further clarifications on what the project is structured to provide,
//! we consider "solutions" which:
//!
//! - Are weak or strong
//! - Start from an arbitrary state in the game
//! - Are equilibrium concepts (for probabilistic games)
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

/* SOLVER MODULES */

pub mod stochastic;
pub mod strong;
pub mod weak;

/* CONSTANTS */

/// Describes the maximum number of states that are one move away from any state
/// within a game. Used to allocate statically-sized arrays on the stack for
/// faster execution of solving algorithms.
pub const MAX_TRANSITIONS: usize = 50;
