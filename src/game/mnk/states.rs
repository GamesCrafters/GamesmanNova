//! # MNK State Handling Module
//!
//! TODO

use anyhow::Result;

use crate::game::State;
use crate::game::mnk::Session;

/* ZERO-BY STATE ENCODING */

pub const STATE_DEFAULT: &str = "TODO";
pub const STATE_PATTERN: &str = r"TODO";
pub const STATE_PROTOCOL: &str = "TODO";

/* API */

/// TODO
pub fn parse_state(session: &Session, from: String) -> Result<State> {
    todo!()
}
