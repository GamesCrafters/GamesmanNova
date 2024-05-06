//! # Game Utilities Module
//!
//! This module provides some common utilities used in the implementation of
//! more than a single game.
//!
//! #### Authorship
//! - Max Fierro, 11/2/2023 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};

use crate::{
    game::{error::GameError, Bounded, Codec, Game, Transition},
    model::game::State,
};

/* STATE HISTORY VERIFICATION */

/// Returns the latest state in a sequential `history` of state string encodings
/// by verifying that the first state in the history is the same as the `game`'s
/// start and that each state can be reached from its predecessor through the
/// `game`'s transition function. If these conditions are not met, it returns an
/// error message signaling the pair of states that are not connected by the
/// transition function, with a reminder of the current game variant.
pub fn verify_history_dynamic<const B: usize, G>(
    game: &G,
    history: Vec<String>,
) -> Result<State<B>>
where
    G: Game + Codec<B> + Bounded<B> + Transition<B>,
{
    if let Some(s) = history.first() {
        let mut prev = game.decode(s.clone())?;
        if prev == game.start() {
            for i in 1..history.len() {
                let next = game.decode(history[i].clone())?;
                let transitions = game.prograde(prev);
                if !transitions.contains(&next) {
                    return transition_history_error(game, prev, next);
                }
                prev = next;
            }
            Ok(prev)
        } else {
            start_history_error(game, game.start())
        }
    } else {
        empty_history_error(game)
    }
}

fn empty_history_error<const B: usize, G>(game: &G) -> Result<State<B>>
where
    G: Game + Codec<B>,
{
    Err(GameError::InvalidHistory {
        game_name: game.info().name,
        hint: format!("State history must contain at least one state."),
    })
    .context("Invalid game history.")
}

fn start_history_error<const B: usize, G>(
    game: &G,
    start: State<B>,
) -> Result<State<B>>
where
    G: Game + Codec<B>,
{
    Err(GameError::InvalidHistory {
        game_name: game.info().name,
        hint: format!(
            "The state history must begin with the starting state for this \
            variant ({}), which is {}.",
            game.info().variant,
            game.encode(start)
        ),
    })
    .context("Invalid game history.")
}

fn transition_history_error<const B: usize, G>(
    game: &G,
    prev: State<B>,
    next: State<B>,
) -> Result<State<B>>
where
    G: Game + Codec<B>,
{
    Err(GameError::InvalidHistory {
        game_name: game.info().name,
        hint: format!(
            "Transitioning from the state '{}' to the sate '{}' is \
            illegal in the current game variant ({}).",
            game.encode(prev),
            game.encode(next),
            game.info().variant
        ),
    })
    .context("Invalid game history.")
}
