//! # Game Utilities Module
//!
//! This module provides some common utilities used in the implementation of
//! more than a single game.
//!
//! #### Authorship
//!
//! - Max Fierro, 11/2/2023 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};

use crate::{
    game::error::GameError,
    game::{DTransition, Legible, STransition},
    model::{PlayerCount, State, Turn},
    solver::MAX_TRANSITIONS,
};

/* TURN ENCODING */

/// Minimally encodes turn information into the 64-bit integer `state` by
/// shifting the integer in `state` just enough bits to allow `turn` to be
/// expressed, where `turn` is upper-bounded by `player_count`.
///
/// For example, if `player_count` is 3, `state` is `0b00...01`, and we want to
/// encode that it is player `2`'s turn (where players are 0-indexed), we would
/// return `0b00...00111`, whereas if `player_count` was 2 we would return
/// `0b00...0011`. This is because you need two bits to enumerate `{0, 1, 2}`,
/// but only one to enumerate `{0, 1}`.
pub fn pack_turn(state: State, turn: Turn, player_count: PlayerCount) -> State {
    if player_count == 0 {
        return state;
    } else {
        let turn_bits = Turn::BITS - (player_count - 1).leading_zeros();
        (state << turn_bits) | (turn as State)
    }
}

/// Given a state and a player count, determines the player whose turn it is by
/// taking note of the integer in the rightmost bits of `state`. The number of
/// bits considered turn information are determined by `player_count`. This is
/// the inverse function of `pack_turn`.
pub fn unpack_turn(
    encoding: State,
    player_count: PlayerCount,
) -> (State, Turn) {
    if player_count == 0 {
        return (encoding, 0);
    } else {
        let turn_bits = Turn::BITS - (player_count - 1).leading_zeros();
        let turn_mask = (1 << turn_bits) - 1;
        let state = (encoding & !turn_mask) >> turn_bits;
        let turn = (encoding & turn_mask) as usize;
        (state, turn)
    }
}

/* STATE HISTORY VERIFICATION */

/// Returns the latest state in a sequential `history` of state string encodings
/// by verifying that the first state in the history is the same as the `game`'s
/// start and that each state can be reached from its predecessor through the
/// `game`'s transition function. If these conditions are not met, it returns an
/// error message signaling the pair of states that are not connected by the
/// transition function, with a reminder of the current game variant.
pub fn verify_history_dynamic<G>(
    game: &G,
    history: Vec<String>,
) -> Result<State>
where
    G: Legible<State> + DTransition<State>,
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

/// Returns the latest state in a sequential `history` of state string encodings
/// by verifying that the first state in the history is the same as the `game`'s
/// start and that each state can be reached from its predecessor through the
/// `game`'s transition function. If these conditions are not met, it returns an
/// error message signaling the pair of states that are not connected by the
/// transition function, with a reminder of the current game variant.
pub fn verify_history_static<G>(game: &G, history: Vec<String>) -> Result<State>
where
    G: Legible<State> + STransition<State, MAX_TRANSITIONS>,
{
    if let Some(s) = history.first() {
        let mut prev = game.decode(s.clone())?;
        if prev == game.start() {
            for i in 1..history.len() {
                let next = game.decode(history[i].clone())?;
                let transitions = game.prograde(prev);
                if !transitions.contains(&Some(next)) {
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

fn empty_history_error<G: Legible<State>>(game: &G) -> Result<State> {
    Err(GameError::InvalidHistory {
        game_name: game.info().name,
        hint: format!("State history must contain at least one state."),
    })
    .context("Invalid game history.")
}

fn start_history_error<G: Legible<State>>(
    game: &G,
    start: State,
) -> Result<State> {
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

fn transition_history_error<G: Legible<State>>(
    game: &G,
    prev: State,
    next: State,
) -> Result<State> {
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