//! # Game Utilities Module
//!
//! This module provides some common utilities used in the implementation of
//! more than a single game.
//!
//! #### Authorship
//! - Max Fierro, 11/2/2023 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};

use crate::game::Information;
use crate::{
    game::{error::GameError, Bounded, Codec, Transition},
    model::game::State,
};

/* STATE HISTORY VERIFICATION */

pub fn verify_state_history<const B: usize, G>(
    game: &G,
    history: Vec<String>,
) -> Result<State<B>>
where
    G: Information + Bounded<B> + Codec<B> + Transition<B>,
{
    if let Some(s) = history.first() {
        let mut prev = game.decode(s.clone())?;
        if prev == game.start() {
            for i in 1..history.len() {
                let next = game.decode(history[i].clone())?;
                let transitions = game.prograde(prev);
                if !transitions.contains(&next) {
                    return transition_history_error(game, prev, next)
                        .context("Specified invalid state transition.");
                }
                prev = next;
            }
            Ok(prev)
        } else {
            start_history_error(game, game.start())
                .context("Specified invalid first state.")
        }
    } else {
        empty_history_error::<B, G>()
            .context("Provided state history is empty.")
    }
}

/* HISTORY VERIFICATION ERRORS */

fn empty_history_error<const B: usize, G>() -> Result<State<B>>
where
    G: Information + Codec<B> + Bounded<B>,
{
    Err(GameError::InvalidHistory {
        game_name: G::info().name,
        hint: format!("State history must contain at least one state."),
    })?
}

fn start_history_error<const B: usize, G>(
    game: &G,
    start: State<B>,
) -> Result<State<B>>
where
    G: Information + Codec<B> + Bounded<B>,
{
    Err(GameError::InvalidHistory {
        game_name: G::info().name,
        hint: format!(
            "The state history must begin with the starting state for the \
            provided game variant, which is {}.",
            game.encode(start)?
        ),
    })?
}

fn transition_history_error<const B: usize, G>(
    game: &G,
    prev: State<B>,
    next: State<B>,
) -> Result<State<B>>
where
    G: Information + Codec<B> + Bounded<B>,
{
    Err(GameError::InvalidHistory {
        game_name: G::info().name,
        hint: format!(
            "Transitioning from the state '{}' to the sate '{}' is illegal in \
            the provided game variant.",
            game.encode(prev)?,
            game.encode(next)?,
        ),
    })?
}
