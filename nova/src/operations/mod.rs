//! # Execution Handling Moudle
//!
//! This module handles requests made through specific commands.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

/* MODULES */

/// Request handling module for `nova analyze` commands.
pub mod analyzing;

/// Request handling module for `nova tui` (and eventually `nova daemon`) commands.
pub mod interfacing;

/// Request handling module for `nova list` commands.
pub mod listing;

/// Request handling module for `nova solve` commands.
pub mod solving;
