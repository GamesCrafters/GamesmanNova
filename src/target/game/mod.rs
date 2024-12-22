//! # Game Target Module
//!
//! This module provides implementations of various sequential games to be used
//! as feature extraction targets.

#[cfg(test)]
pub mod test;
pub mod util;
pub mod model;

/* IMPLEMENTED GAME MODULES */

#[cfg(test)]
pub mod mock;

pub mod crossteaser;
pub mod zero_by;
