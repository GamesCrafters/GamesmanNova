//! # Solving Execution Module
//!
//! This module contains handling behavior for all `nova solve` requests.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::errors::NovaError;
use crate::interfaces::find_game;
use crate::interfaces::terminal::cli::SolveArgs;

/// Attempts to solve the game with the indicated `name`, and returns the value
/// or an error containing what was actually passed in versus what was
/// probably meant to be passed in.
pub fn solve_by_name(args: &SolveArgs) -> Result<(), NovaError> {
    let game = find_game(args.target, args.variant.clone())?;
    game.solve(args.mode)?;
    Ok(())
}
