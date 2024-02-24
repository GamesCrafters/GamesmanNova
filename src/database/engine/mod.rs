//! # Database Engine Module
//!
//! This module has all available implementations for databases that adhere to
//! one or more of the interfaces of the `crate::database` module. These modules
//! are meant to be referenced directly from solvers, depending on which one
//! they would benefit from the most.
//!
//! #### Authorship
//!
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

pub mod volatile;
pub mod simple;
pub mod lsmt;
