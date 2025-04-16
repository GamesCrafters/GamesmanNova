//! # Solver Utilities Module
//!
//! This module makes room for common utility routines used throughout the
//! `crate::solver` module.

use anyhow::Result;

use std::ops::Not;

use crate::solver::IUtility;
use crate::solver::SUtility;
use crate::solver::error::SolverError;

/* UTILITIES */

/// Transform input string into a valid SQL identifier.
pub fn sqlize(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i == 0 {
            if ch.is_ascii_alphabetic() || ch == '_' {
                out.push(ch);
            } else {
                out.push('_');
            }
        } else if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    out
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
