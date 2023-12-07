//! # Weak Solving Module
//!
//! This module implements weak solvers for all applicable types of games
//! through blanket implementations. Solvers that take advantage of different
//! game structures are under modules whose names describe what that structure
//! is. For example, the blanket implementation for `weak::acyclic::Solver`
//! should provide a weak solver for all `Acyclic` games through a blanket
//! implementation.
//!
//! #### Authorship
//!
//! - Max Fierro, 12/4/2023 (maxfierro@berkeley.edu)

/// # Weak Acyclic Solving Module
///
/// This module implements weak acyclic solvers for all applicable types of
/// games through blanket implementations of the `acyclic::Solver` trait,
/// optimizing for specific game characteristics wherever possible.
///
/// #### Authorship
///
/// - Max Fierro, 12/4/2023 (maxfierro@berkeley.edu)
pub mod acyclic {}

/// # Weak Cyclic Solving Module
///
/// This module implements weak cyclic solvers for all applicable types of
/// games through blanket implementations of the `acyclic::Solver` trait,
/// optimizing for specific game characteristics wherever possible.
///
/// #### Authorship
///
/// - Max Fierro, 12/4/2023 (maxfierro@berkeley.edu)
pub mod cyclic {}
