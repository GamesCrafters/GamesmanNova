//! # Execution Handling Moudle
//!
//! This module handles requests made through specific commands.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

// use crate::core::solvers::Solvable;
// use crate::{core::Variant, games::Game};

/* MODULES */

/// Request handling module for `nova analyze` commands.
pub mod analyzing;

/// Request handling module for `nova tui` (and eventually `nova daemon`) commands.
pub mod interfacing;

/// Request handling module for `nova list` commands.
pub mod listing;

/// Request handling module for `nova solve` commands.
pub mod solving;

// fn a() {
//     let target = "zero_by";
//     let session = get_session::generate_match!("src/games/")(None);

// }
