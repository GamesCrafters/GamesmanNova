//! # Interfaces Library
//!
//! `interfaces` provides all the available ways to interact with GamesmanNova.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::{
    errors::NovaError,
    games::{crossteaser, zero_by, Game},
    models::Variant,
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
    Crossteaser,
}

/// Fetches and initializes the correct game session based on an indicated
/// `GameModule`, with the provided `variant`.
pub fn find_game(
    game: GameModule,
    variant: Option<Variant>,
) -> Result<Box<dyn Game>, NovaError> {
    match game {
        GameModule::ZeroBy => {
            Ok(Box::new(zero_by::Session::initialize(variant)?))
        }
        GameModule::Crossteaser => {
            Ok(Box::new(crossteaser::Session::initialize(variant)?))
        }
    }
}
