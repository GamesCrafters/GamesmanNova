//! # Zero-By Utility Module
//!
//! This module provides some common utilities used in the implementation of the
//! game Zero-By, such as bit encoding and decoding.
//!
//! #### Authorship
//!
//! - Max Fierro, 11/2/2023 (maxfierro@berkeley.edu)

use crate::models::{Player, State};

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
pub fn pack_turn(state: State, turn: Player, player_count: Player) -> State
{
    let turn_bits = Player::BITS - (player_count - 1).leading_zeros();
    let shifted_state = state << turn_bits;
    shifted_state + turn
}

/// Given a state and a player count, determines the player whose turn it is by
/// taking note of the integer in the rightmost bits of `state`. The number of
/// bits considered turn information are determined by `player_count`. This is
/// the inverse function of `pack_turn`.
pub fn unpack_turn(encoding: State, player_count: Player) -> (State, Player)
{
    let turn_bits = Player::BITS - (player_count - 1).leading_zeros();
    let state_bits = State::BITS - turn_bits;
    let turn_u: Player = (encoding << state_bits) >> state_bits;
    let state_u: State = encoding >> turn_bits;
    (state_u, turn_u)
}

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn pack_turn_correctness()
    {
        // Require three turn bits (8 players = {0b000, 0b001, ..., 0b111})
        let player_count: Player = 8;
        // 5 in decimal
        let turn: Player = 0b0000_0101;
        // 31 in decimal
        let state: State = 0b0001_1111;
        // 0b00...00_1111_1101 in binary, 0x00...00FC in hexadecimal
        assert!(0xfc == pack_turn(state, turn, player_count));
    }

    #[test]
    fn unpack_turn_correctness()
    {
        // Require six turn bits (players = {0b0, 0b1, ..., 0b100101})
        let player_count: Player = 38;
        // 346 in decimal
        let encoding: State = 0b0001_0101_1010;
        // 0b00...00_0001_0101_1010 -> 0b00...00_0101 and 0b0001_1010, which
        // means that 346 should be decoded to a state of 5 and a turn of 26
        assert!((5, 26) == unpack_turn(encoding, player_count));
    }

    #[test]
    fn unpack_is_inverse_of_pack()
    {
        // Require two turn bits (players = {0b00, 0b01, 0b10})
        let player_count: Player = 3;
        // 0b00...01 in binary
        let turn: Player = 2;
        // 0b00...0111 in binary
        let state: State = 7;
        // 0b00...011101 in binary
        let packed: State = pack_turn(state, turn, player_count);
        // Packing and unpacking should yield equivalent results
        assert!((state, turn) == unpack_turn(packed, player_count));

        let prime = 97;

        // About 255 * prime^2 iterations
        for p in Player::MIN..=Player::MAX {
            let turn_bits = Player::BITS - p.leading_zeros();
            let max_state: State = State::MAX / (2 ^ turn_bits);
            for s in (State::MIN..max_state).step_by(max_state / prime) {
                for t in (Player::MIN..=p).step_by(p / prime) {
                    assert!((s, t) == unpack_turn(pack_turn(s, t, p), p));
                }
            }
        }
    }
}
