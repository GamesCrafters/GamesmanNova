//! # MNK Game Feature Calculators
//!
//! Elements that compute attributes about m,n,k game states.

use crate::game::State;
use crate::game::mnk::Board;
use crate::game::mnk::Session;
use crate::game::mnk::Symbol;

impl Session {
    /// [GPT] Return true if the side to move has a “fork,” i.e. some move that
    /// creates two (or more) immediate winning threats. Here, true=1, false=-1.
    pub fn fork_exists(&self, state: State) -> i64 {
        let (turn, board) = self.decode_state(state);
        let me = if turn == 1 { Symbol::X } else { Symbol::O };
        let k = self.k;

        // helper: does placing `me` at (i,j) win immediately?
        let is_win = |b: &Board, i: usize, j: usize| -> bool {
            // count in one direction + the opposite
            let count_dir = |di: isize, dj: isize| {
                let mut cnt = 0;
                let mut r = i as isize + di;
                let mut c = j as isize + dj;
                while r >= 0
                    && r < self.m as isize
                    && c >= 0
                    && c < self.n as isize
                    && b[r as usize][c as usize] == me
                {
                    cnt += 1;
                    r += di;
                    c += dj;
                }
                cnt
            };

            for &((dx1, dy1), (dx2, dy2)) in &[
                ((0, 1), (0, -1)),
                ((1, 0), (-1, 0)),
                ((1, 1), (-1, -1)),
                ((1, -1), (-1, 1)),
            ] {
                let total = 1 + count_dir(dx1, dy1) + count_dir(dx2, dy2);
                if total >= k {
                    return true;
                }
            }
            false
        };

        // for each empty cell, simulate your move and count winning replies
        for i in 0..self.m {
            for j in 0..self.n {
                if board[i][j] != Symbol::B {
                    continue;
                }

                let mut b2 = board;
                b2[i][j] = me;

                let mut threats = 0;
                for ii in 0..self.m {
                    for jj in 0..self.n {
                        if b2[ii][jj] == Symbol::B && is_win(&b2, ii, jj) {
                            threats += 1;
                            if threats >= 2 {
                                return 1;
                            }
                        }
                    }
                }
            }
        }

        -1
    }

    /// [GPT] Number of moves already played (non‐blank cells)
    pub fn ply(&self, state: State) -> i64 {
        let (_turn, board) = self.decode_state(state);
        let mut count = 0;
        (0..self.m).for_each(|i| {
            for j in 0..self.n {
                if board[i][j] != Symbol::B {
                    count += 1;
                }
            }
        });
        count
    }

    /// [GPT]
    /// +1 if the center cell is occupied by the side to move,
    /// -1 if occupied by opponent, 0 if empty or no exact center.
    pub fn center_control(&self, state: State) -> i64 {
        let (turn, board) = self.decode_state(state);
        let me = if turn == 1 { Symbol::X } else { Symbol::O };
        let opp = if me == Symbol::X { Symbol::O } else { Symbol::X };

        // only defined when both dims are odd
        if self.m % 2 == 1 && self.n % 2 == 1 {
            let ci = self.m / 2;
            let cj = self.n / 2;
            return match board[ci][cj] {
                b if b == me => 1,
                b if b == opp => -1,
                _ => 0,
            };
        }
        0
    }

    /// [GPT]
    /// Returns (your corners) – (opponent corners)
    /// corners = (0,0), (0,n-1), (m-1,0), (m-1,n-1)
    pub fn corner_count(&self, state: State) -> i64 {
        let (turn, board) = self.decode_state(state);
        let me = if turn == 1 { Symbol::X } else { Symbol::O };
        let opp = if me == Symbol::X { Symbol::O } else { Symbol::X };

        let mut diff = 0;
        let last_row = self.m - 1;
        let last_col = self.n - 1;
        let corners = [
            (0, 0),
            (0, last_col),
            (last_row, 0),
            (last_row, last_col),
        ];

        for &(i, j) in &corners {
            diff += match board[i][j] {
                b if b == me => 1,
                b if b == opp => -1,
                _ => 0,
            };
        }
        diff
    }

    /// [GPT]
    /// Returns (your edges) – (opponent edges)
    /// edges = border cells excluding the four corners
    pub fn edge_count(&self, state: State) -> i64 {
        let (turn, board) = self.decode_state(state);
        let me = if turn == 1 { Symbol::X } else { Symbol::O };
        let opp = if me == Symbol::X { Symbol::O } else { Symbol::X };

        let mut diff = 0;
        let last_row = self.m - 1;
        let last_col = self.n - 1;

        // top & bottom row (excluding corners)
        if self.n > 2 {
            for j in 1..last_col {
                diff += match board[0][j] {
                    b if b == me => 1,
                    b if b == opp => -1,
                    _ => 0,
                };
                diff += match board[last_row][j] {
                    b if b == me => 1,
                    b if b == opp => -1,
                    _ => 0,
                };
            }
        }

        // left & right column (excluding corners)
        if self.m > 2 {
            (1..last_row).for_each(|i| {
                diff += match board[i][0] {
                    b if b == me => 1,
                    b if b == opp => -1,
                    _ => 0,
                };
                diff += match board[i][last_col] {
                    b if b == me => 1,
                    b if b == opp => -1,
                    _ => 0,
                };
            });
        }

        diff
    }
}
