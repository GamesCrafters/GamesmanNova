//! # Extensive Game State Handling Module
//!
//! This module helps parse the a string encoding of an abstract extensive game
//! state into a more efficient binary representation, performing a series of
//! checks which ensure compatibility with a game instance.
//!
//! #### Authorship
//!
//! - Max Fierro, 3/31/2024 (maxfierro@berkeley.edu)

/* MOCK STATE ENCODING */

pub const STATE_DEFAULT: &'static str = "N/A";
pub const STATE_PATTERN: &'static str = r"^\d+$";
pub const STATE_PROTOCOL: &'static str = "A non-negative integer.";
