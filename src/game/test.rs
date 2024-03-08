//! # Game Test Module
//!
//! This module provides some unit tests used in the implementation of
//! more than a single game.
//!
//! #### Authorship
//!
//! - Max Fierro, 11/2/2023 (maxfierro@berkeley.edu)
//! - Benjamin Riley Zimmerman, 3/8/2024 (bz931@berkely.edu)

/* TESTS */

#[cfg(test)]
mod test {

    use crate::*;

    /* TURN ENCODING TESTS */

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
