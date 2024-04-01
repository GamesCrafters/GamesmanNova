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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn pack_turn_correctness() {
        // Require three turn bits (8 players = {0b000, 0b001, ..., 0b111})
        let player_count: Turn = 8;
        // 5 in decimal
        let turn: Turn = 0b0000_0101;
        // 31 in decimal
        let state: State = 0b0001_1111;
        // 0b00...00_1111_1101 in binary = 0b[state bits][player bits]
        assert_eq!(0b1111_1101, pack_turn(state, turn, player_count));
    }

    #[test]
    fn unpack_turn_correctness() {
        // Require six turn bits (players = {0b0, 0b1, ..., 0b100101})
        let player_count: Turn = 38;
        // 346 in decimal
        let encoding: State = 0b0001_0101_1010;
        // 0b00...00_0001_0101_1010 -> 0b00...00_0101 and 0b0001_1010, which
        // means that 346 should be decoded to a state of 5 and a turn of 26
        assert_eq!((5, 26), unpack_turn(encoding, player_count));
    }

    #[test]
    fn unpack_is_inverse_of_pack() {
        // Require two turn bits (players = {0b00, 0b01, 0b10})
        let player_count: Turn = 3;
        // 0b00...01 in binary
        let turn: Turn = 2;
        // 0b00...0111 in binary
        let state: State = 7;
        // 0b00...011101 in binary
        let packed: State = pack_turn(state, turn, player_count);
        // Packing and unpacking should yield equivalent results
        assert_eq!((state, turn), unpack_turn(packed, player_count));

        // About 255 * 23^2 iterations
        for p in Turn::MIN..=255 {
            let turn_bits = Turn::BITS - p.leading_zeros();
            let max_state: State = State::MAX / ((1 << turn_bits) as u64);
            let state_step = ((max_state / 23) + 1) as usize;
            let turn_step = ((p / 23) + 1) as usize;

            for s in (State::MIN..max_state).step_by(state_step) {
                for t in (Turn::MIN..p).step_by(turn_step) {
                    assert_eq!((s, t), unpack_turn(pack_turn(s, t, p), p));
                }
            }
        }
    }
}
