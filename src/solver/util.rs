//! # Solver Utilities Module
//!
//! This module makes room for common utility routines used throughout the
//! `crate::solver` module.
//!
//! #### Authorship
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use std::fmt::Display;
use std::ops::Not;

use crate::database::Schema;
use crate::model::solver::{IUtility, RUtility, SUtility};
use crate::solver::error::SolverError;
use crate::solver::{record, RecordType};

/* RECORD TYPE IMPLEMENTATIONS */

impl Display for RecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordType::MUR(players) => {
                write!(f, "Real Utility Remoteness ({players} players)")
            },
            RecordType::SUR(players) => {
                write!(
                    f,
                    "Simple Utility Remoteness ({players}  players)",
                )
            },
            RecordType::SURCC(players) => {
                write!(
                    f,
                    "Simple Utility Remoteness with Child Count ({}  players)",
                    players
                )
            },
            RecordType::REM => {
                write!(f, "Remoteness Only")
            },
        }
    }
}

impl TryInto<Schema> for RecordType {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Schema, Self::Error> {
        match self {
            RecordType::SURCC(players) => record::surcc::schema(players),
            RecordType::MUR(players) => record::mur::schema(players),
            RecordType::SUR(players) => record::sur::schema(players),
            RecordType::REM => record::rem::schema(),
        }
    }
}

/* CONVERSIONS INTO SIMPLE UTILITY */

impl TryFrom<IUtility> for SUtility {
    type Error = SolverError;

    fn try_from(v: IUtility) -> Result<Self, Self::Error> {
        match v {
            v if v == SUtility::Lose as i64 => Ok(SUtility::Lose),
            v if v == SUtility::Draw as i64 => Ok(SUtility::Draw),
            v if v == SUtility::Tie as i64 => Ok(SUtility::Tie),
            v if v == SUtility::Win as i64 => Ok(SUtility::Win),
            _ => Err(SolverError::InvalidConversion {
                input_t: "Integer Utility".into(),
                output_t: "Simple Utility".into(),
                hint:
                    "Down-casting from integer to simple utility values is not \
                    stable, and relies on the internal representation used for \
                    simple utility values (which is not intuitive)."
                        .into(),
            }),
        }
    }
}

impl TryFrom<RUtility> for SUtility {
    type Error = SolverError;

    fn try_from(v: RUtility) -> Result<Self, Self::Error> {
        match v {
            v if v as i64 == SUtility::Lose as i64 => Ok(SUtility::Lose),
            v if v as i64 == SUtility::Draw as i64 => Ok(SUtility::Draw),
            v if v as i64 == SUtility::Tie as i64 => Ok(SUtility::Tie),
            v if v as i64 == SUtility::Win as i64 => Ok(SUtility::Win),
            _ => Err(SolverError::InvalidConversion {
                input_t: "Real Utility".into(),
                output_t: "Simple Utility".into(),
                hint: "Down-casting from real-valued to simple utility values \
                    is not stable, and relies on the internal representation \
                    used for simple utility values (which is not intuitive)."
                    .into(),
            }),
        }
    }
}

impl TryFrom<u64> for SUtility {
    type Error = SolverError;

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            v if v as i64 == SUtility::Lose as i64 => Ok(SUtility::Lose),
            v if v as i64 == SUtility::Draw as i64 => Ok(SUtility::Draw),
            v if v as i64 == SUtility::Tie as i64 => Ok(SUtility::Tie),
            v if v as i64 == SUtility::Win as i64 => Ok(SUtility::Win),
            _ => Err(SolverError::InvalidConversion {
                input_t: "u64".into(),
                output_t: "Simple Utility".into(),
                hint: "Down-casting from integer to simple utility values \
                    is not stable, and relies on the internal representation \
                    used for simple utility values (which is not intuitive)."
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
            SUtility::Draw => 0,
            SUtility::Tie => 0,
            SUtility::Win => 1,
        }
    }
}

impl From<SUtility> for RUtility {
    fn from(v: SUtility) -> Self {
        match v {
            SUtility::Lose => -1.0,
            SUtility::Draw => 0.0,
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
            SUtility::Draw => SUtility::Draw,
            SUtility::Lose => SUtility::Win,
            SUtility::Win => SUtility::Lose,
            SUtility::Tie => SUtility::Tie,
        }
    }
}
