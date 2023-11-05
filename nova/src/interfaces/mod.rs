//! # Interfaces Library
//!
//! `interfaces` provides all the available ways to interact with GamesmanNova.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::{
    errors::VariantError,
    games::{zero_by, Game, Solvable},
    models::Variant,
};
use clap::ValueEnum;

/* MODULES */

pub mod graphical;
pub mod networked;
pub mod terminal;

/// Specifies the game offerings available through all interfaces.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum GameModule
{
    ZeroBy,
}

/// Fetches and initializes the correct game session based on an indicated
/// `GameModule`, with the provided `variant`.
pub fn find_game_2(
    game: GameModule,
    variant: Option<Variant>,
) -> Result<impl Solvable<2>, VariantError>
{
    match game {
        GameModule::ZeroBy => Ok(zero_by::Session::initialize(variant)?),
    }
}
