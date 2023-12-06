//! # Interfaces Library
//!
//! This module provides all the available behavior used to interact with the
//! project through different ways, such as the command-line.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::{
    errors::NovaError,
    games::{zero_by, Game},
};
use clap::ValueEnum;

/* MODULES */

pub mod graphical;
pub mod networked;
pub mod terminal;

/// Specifies the game offerings available through all interfaces.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum GameModule {
    ZeroBy,
}

/// Fetches and initializes the correct game session based on an indicated
/// `GameModule`, with the provided `variant`.
pub fn find_game(
    game: GameModule,
    variant: Option<String>,
    state: Option<String>,
) -> Result<Box<dyn Game>, NovaError> {
    match game {
        GameModule::ZeroBy => Ok(Box::new(zero_by::Session::initialize(
            variant, state,
        )?)),
    }
}
