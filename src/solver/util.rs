//! # Solver Utilities Module
//!
//! This module makes room for common utility routines used throughout the
//! `crate::solver` module.

use std::ops::Not;

use crate::solver::error::SolverError;
use crate::solver::model::{IUtility, RUtility, SUtility};
use crate::solver::SOLUTION_TABLE_POSTFIX;

/* FILENAMES */

/// Return a standard name used for the namespace associated with a solution for
/// the provided `game`.
pub fn solution_namespace(game: &str) -> String {
    format!("{}_{}", game, SOLUTION_TABLE_POSTFIX)
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

impl TryFrom<RUtility> for SUtility {
    type Error = SolverError;

    fn try_from(v: RUtility) -> Result<Self, Self::Error> {
        match v {
            _ if v as i8 == SUtility::Lose as i8 => Ok(SUtility::Lose),
            _ if v as i8 == SUtility::Tie as i8 => Ok(SUtility::Tie),
            _ if v as i8 == SUtility::Win as i8 => Ok(SUtility::Win),
            _ => Err(SolverError::InvalidConversion {
                input_t: "Real Utility".into(),
                output_t: "Simple Utility".into(),
                hint: "Down-casting from real-valued to simple utility values \
                    is not stable, and relies on the internal representation \
                    used for simple utility values."
                    .into(),
            }),
        }
    }
}

impl TryFrom<i8> for SUtility {
    type Error = SolverError;

    fn try_from(v: i8) -> Result<Self, Self::Error> {
        match v {
            _ if v as i8 == SUtility::Lose as i8 => Ok(SUtility::Lose),
            _ if v as i8 == SUtility::Tie as i8 => Ok(SUtility::Tie),
            _ if v as i8 == SUtility::Win as i8 => Ok(SUtility::Win),
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

impl From<SUtility> for RUtility {
    fn from(v: SUtility) -> Self {
        match v {
            SUtility::Lose => -1.0,
            SUtility::Tie => 0.0,
            SUtility::Win => 1.0,
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
