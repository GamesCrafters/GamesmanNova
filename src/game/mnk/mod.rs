//! # MNK Game Module
//!
//! The m,n,k game is a generalization of Tic-Tac-Toe that is also acyclic. It
//! allows for play on an m-by-n board, where k symbols in a row belonging to
//! either of the two players results in an immediate win for that player.

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use bitvec::array::BitArray;
use bitvec::field::BitField;
use bitvec::order::Msb0;
use rusqlite::Error::QueryReturnedNoRows;
use rusqlite::Statement;
use rusqlite::Transaction;
use rusqlite::params_from_iter;

use crate::game::Codec;
use crate::game::Forward;
use crate::game::GameData;
use crate::game::Implicit;
use crate::game::Information;
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::State;
use crate::game::Variable;
use crate::game::Variant;
use crate::game::mnk::states::*;
use crate::game::mnk::variants::*;
use crate::interface::IOMode;
use crate::solver::Game;
use crate::solver::Persistent;
use crate::solver::Queries;
use crate::solver::SUtility;
use crate::solver::SimpleUtility;
use crate::solver::Solution;
use crate::solver::algorithm::acyclic;
use crate::solver::db::Schema;

/* SUBMODULES */

mod states;
mod variants;

/* DEFINITIONS */

type Board = [[Symbol; MAX_BOARD_SIDE]; MAX_BOARD_SIDE];

const MAX_BOARD_SIDE: usize = 10;

#[derive(Clone, Copy, PartialEq)]
enum Symbol {
    B = 0,
    X = 1,
    O = 2,
}

/* GAME DATA */

const NAME: &str = "mnk";
const AUTHORS: &str = "Max Fierro <maxfierro@berkeley.edu>";
const ABOUT: &str = "Two players take turns placing Xs and Os on a square grid \
of dimensions MxN. The first player to complete K of their own symbol in a \
row, which may be diagonally, horizontally, or vertically, wins the game. \
Skipping moves is not allwed; players must place a symbol on their turn.";

/* GAME IMPLEMENTATION */

pub struct Session {
    schema: Schema,
    start: State,
    m: usize,
    n: usize,
    k: usize,
}

impl Session {
    pub fn new(variant: Option<Variant>) -> Result<Self> {
        if let Some(v) = variant {
            Self::variant(v)
        } else {
            Ok(Self::default())
        }
    }

    pub fn solve(&mut self, mode: IOMode) -> Result<()> {
        acyclic::solve::<2, 8, _>(self, mode)
    }

    /* INTERNAL API */

    fn encode_state(&self, turn: Player, board: &Board) -> State {
        let mut state = BitArray::<_, Msb0>::ZERO;
        (0..self.m).for_each(|i| {
            for j in 0..self.n {
                let cell = i * self.n + j;
                let start = 1 + 2 * cell;
                state[start..start + 2].store_be(board[i][j] as u8);
            }
        });

        state[..1].store_be(turn);
        state.data
    }

    fn decode_state(&self, state: State) -> (Player, Board) {
        let state = BitArray::<[u8; 8], Msb0>::from(state);
        let turn = state[..1].load_be::<Player>();
        let mut board = [[Symbol::B; MAX_BOARD_SIDE]; MAX_BOARD_SIDE];
        (0..self.m).for_each(|i| {
            for j in 0..self.n {
                let start = 1 + 2 * (i * self.n + j);
                let code = state[start..start + 2].load_be::<u8>();
                board[i][j] = Symbol::from(code);
            }
        });

        (turn, board)
    }

    fn win(&self, board: &Board, sym: Symbol) -> bool {
        (0..self.m).any(|i| {
            (0..=(self.n - self.k))
                .any(|j| (0..self.k).all(|d| board[i][j + d] == sym))
        }) || (0..self.n).any(|j| {
            (0..=(self.m - self.k))
                .any(|i| (0..self.k).all(|d| board[i + d][j] == sym))
        }) || (0..=(self.m - self.k)).any(|i| {
            (0..=(self.n - self.k))
                .any(|j| (0..self.k).all(|d| board[i + d][j + d] == sym))
        }) || ((self.k - 1)..self.m).any(|i| {
            (0..=(self.n - self.k))
                .any(|j| (0..self.k).all(|d| board[i - d][j + d] == sym))
        })
    }

    fn draw(&self, board: &Board) -> bool {
        (0..self.m).all(|i| (0..self.n).all(|j| board[i][j] != Symbol::B))
    }
}

/* IMPLEMENTATIONS */

impl Default for Session {
    fn default() -> Self {
        parse_variant(VARIANT_DEFAULT.to_owned())
            .expect("Failed to parse default variant.")
    }
}

impl Information for Session {
    fn info() -> GameData {
        GameData {
            name: NAME,
            authors: AUTHORS,
            about: ABOUT,

            variant_protocol: VARIANT_PROTOCOL,
            variant_pattern: VARIANT_PATTERN,
            variant_default: VARIANT_DEFAULT,

            state_pattern: STATE_PATTERN,
            state_default: STATE_DEFAULT,
            state_protocol: STATE_PROTOCOL,
        }
    }
}

impl Variable for Session {
    fn variant(variant: Variant) -> Result<Self> {
        parse_variant(variant).context("Malformed game variant.")
    }
}

impl Implicit for Session {
    fn adjacent(&self, state: State) -> Vec<State> {
        let (turn, board) = self.decode_state(state);
        let sym = if turn == 1 { Symbol::X } else { Symbol::O };
        let next = 1 - turn;
        let mut out = Vec::new();
        for i in 0..self.m {
            for j in 0..self.n {
                if board[i][j] == Symbol::B {
                    let mut nb = board;
                    nb[i][j] = sym;
                    out.push(self.encode_state(next, &nb));
                }
            }
        }
        out
    }

    fn source(&self) -> State {
        let board = [[Symbol::B; MAX_BOARD_SIDE]; MAX_BOARD_SIDE];
        self.encode_state(1, &board)
    }

    fn sink(&self, state: State) -> bool {
        let (_, board) = self.decode_state(state);
        self.win(&board, Symbol::O)
            || self.win(&board, Symbol::X)
            || self.draw(&board)
    }
}

impl Codec for Session {
    fn decode(&self, string: String) -> Result<State> {
        decode_state_string(self, string)
    }

    fn encode(&self, state: State) -> Result<String> {
        let (_turn, board) = self.decode_state(state);
        encode_state_string(self, &board)
    }
}

impl Forward for Session {
    fn set_verified_start(&mut self, state: State) {
        self.start = state;
    }
}

impl Game<2> for Session {
    fn turn(&self, state: State) -> Player {
        let (turn, _) = self.decode_state(state);
        turn
    }
}

impl SimpleUtility<2> for Session {
    fn utility(&self, state: State) -> [SUtility; 2] {
        let (_turn, board) = self.decode_state(state);
        let mut result = [SUtility::Tie; 2];
        if !self.draw(&board) {
            let x_wins = self.win(&board, Symbol::X);
            let o_wins = self.win(&board, Symbol::O);
            if x_wins && !o_wins {
                result[1] = SUtility::Win;
                result[0] = SUtility::Lose;
            } else if o_wins && !x_wins {
                result[0] = SUtility::Win;
                result[1] = SUtility::Lose;
            } else {
                panic!()
            }
        }

        result
    }
}

impl<const N: PlayerCount> Persistent<N> for Session {
    fn prepare(
        &mut self,
        tx: &mut Transaction,
        mode: IOMode,
    ) -> Result<Queries> {
        let drop_sql = self.schema.drop_table_query();
        let create_sql = self.schema.create_table_query();
        match mode {
            IOMode::Constructive | IOMode::Forgetful => (),
            IOMode::Overwrite => {
                tx.execute(&drop_sql, [])
                    .context("Failed to drop existing table")?;
            },
        }

        tx.execute(&create_sql, [])
            .context("Failed to create table")?;

        let insert = self.schema.insert_query();
        let select = self.schema.select_query();
        let queries = Queries { insert, select };

        Ok(queries)
    }

    fn insert(
        &mut self,
        stmt: &mut Statement,
        state: &State,
        info: &Solution<N>,
    ) -> Result<()> {
        stmt.execute(params_from_iter(
            [
                i64::from_be_bytes(*state),
                info.remoteness as i64,
                info.player as i64,
            ]
            .iter()
            .chain(info.utility.iter()),
        ))?;
        Ok(())
    }

    fn select(
        &mut self,
        stmt: &mut Statement,
        state: &State,
    ) -> Result<Option<Solution<N>>> {
        let start = self.schema.utility_index();
        let row = stmt.query_row([i64::from_be_bytes(*state)], |row| {
            let mut utility: [i64; N] = [0; N];
            for (i, item) in utility.iter_mut().enumerate() {
                *item = row.get(start + i)?;
            }

            Ok(Solution {
                remoteness: row.get(1)?,
                utility,
                player: row.get(2)?,
            })
        });

        match row {
            Err(QueryReturnedNoRows) => Ok(None),
            Ok(data) => Ok(Some(data)),
            Err(e) => Err(anyhow!(e)),
        }
    }
}

/* UTILITY IMPLEMENTATIONS */

impl From<u8> for Symbol {
    fn from(value: u8) -> Self {
        match value {
            0 => Symbol::B,
            1 => Symbol::X,
            2 => Symbol::O,
            other => panic!("Invalid symbol encoding: {}", other),
        }
    }
}
