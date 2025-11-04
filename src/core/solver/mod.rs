//! # Solver Implementations
//!
//! TODO

use anyhow::Result;

use std::ops::Not;

use crate::traits::game::ClassicGame;
use crate::traits::game::ClassicPuzzle;
use crate::traits::game::IntegerUtility;
use crate::traits::game::SimpleUtility;
use crate::types::error::SolverError;
use crate::types::game::IUtility;
use crate::types::game::PlayerCount;
use crate::types::game::SUtility;
use crate::types::game::State;

/* BLANKET IMPLEMENTATIONS */

// All N-player simple-utility games are also N-player integer-utility games.
impl<const N: PlayerCount, const B: usize, G> IntegerUtility<N, B> for G
where
    G: SimpleUtility<N, B>,
{
    fn utility(&self, state: &State<B>) -> [IUtility; N] {
        let sutility = self.utility(state);
        let mut iutility = [0; N];
        iutility
            .iter_mut()
            .enumerate()
            .for_each(|(i, u)| *u = IUtility::from(sutility[i]) - 1);

        iutility
    }
}

// All 2-player zero-sum games are also 2-player simple-utility games.
impl<const B: usize, G> SimpleUtility<2, B> for G
where
    G: ClassicGame<B>,
{
    fn utility(&self, state: &State<B>) -> [SUtility; 2] {
        let mut sutility = [SUtility::Tie; 2];
        let utility = self.utility(state);
        let turn = self.turn(state);
        let them = (turn + 1) % 2;
        sutility[them] = !utility;
        sutility[turn] = utility;
        sutility
    }
}

// All puzzles are also 1-player simple-utility games.
impl<const B: usize, G> SimpleUtility<1, B> for G
where
    G: ClassicPuzzle<B>,
{
    fn utility(&self, state: &State<B>) -> [SUtility; 1] {
        [self.utility(*state)]
    }
}

/* CONVERSIONS INTO SIMPLE UTILITY */

impl TryFrom<IUtility> for SUtility {
    type Error = SolverError;

    fn try_from(v: IUtility) -> Result<Self, Self::Error> {
        match v {
            _ if v == SUtility::Lose as i64 => Ok(SUtility::Lose),
            _ if v == SUtility::Tie as i64 => Ok(SUtility::Tie),
            _ if v == SUtility::Win as i64 => Ok(SUtility::Win),
            _ => Err(SolverError::InvalidConversion {
                input_t: "Integer Utility".into(),
                output_t: "Simple Utility".into(),
                hint:
                    "Down-casting from integer to simple utility values is not \
                    stable, and relies on the internal representation used for \
                    simple utility values."
                        .into(),
            }),
        }
    }
}

impl TryFrom<i8> for SUtility {
    type Error = SolverError;

    fn try_from(v: i8) -> Result<Self, Self::Error> {
        match v {
            _ if v == SUtility::Lose as i8 => Ok(SUtility::Lose),
            _ if v == SUtility::Tie as i8 => Ok(SUtility::Tie),
            _ if v == SUtility::Win as i8 => Ok(SUtility::Win),
            _ => Err(SolverError::InvalidConversion {
                input_t: "i8".into(),
                output_t: "Simple Utility".into(),
                hint: "Down-casting from integer to simple utility values \
                    is not stable, and relies on the internal representation \
                    used for simple utility values."
                    .into(),
            }),
        }
    }
}

/* CONVERSIONS FROM SIMPLE UTILITY */

impl From<SUtility> for IUtility {
    fn from(v: SUtility) -> Self {
        match v {
            SUtility::Lose => -1,
            SUtility::Tie => 0,
            SUtility::Win => 1,
        }
    }
}

/* SIMPLE UTILITY NEGATION */

impl Not for SUtility {
    type Output = SUtility;
    fn not(self) -> Self::Output {
        match self {
            SUtility::Lose => SUtility::Win,
            SUtility::Win => SUtility::Lose,
            SUtility::Tie => SUtility::Tie,
        }
    }
}
