//! # Equilibrium Strategy Stochastic Solving Module
//!
//! This module implements algorithms that solve for optimal strategies that
//! comprise equilibria under different definitions of the term through blanket
//! implementations. Solvers that take advantage of different game  structures
//! are under modules whose names describe what that structure is. For example,
//! the blanket implementation for `stochastic::acyclic::Solver` should provide
//! a stochastic solver for `Acyclic` games through a blanket implementation.
//!
//! #### Authorship
//!
//! - Max Fierro, 12/4/2023 (maxfierro@berkeley.edu)

/// # Stochastic Acyclic Solving Module
///
/// This module contains stochastic acyclic solvers for all applicable types of
/// games through blanket implementations of the `acyclic::Solver` trait,
/// optimizing for specific game characteristics wherever possible.
///
/// #### Authorship
///
/// - Max Fierro, 12/4/2023 (maxfierro@berkeley.edu)
pub mod acyclic {}

/// # Stochastic Cyclic Solving Module
///
/// This module contains stochastic cyclic solvers for all applicable types of
/// games through blanket implementations of the `acyclic::Solver` trait,
/// optimizing for specific game characteristics wherever possible.
///
/// #### Authorship
///
/// - Max Fierro, 12/4/2023 (maxfierro@berkeley.edu)
pub mod cyclic {}
