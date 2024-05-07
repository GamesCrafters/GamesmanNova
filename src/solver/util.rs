//! # Solver Utilities Module
//!
//! This module makes room for common utility routines used throughout the
//! `crate::solver` module.
//!
//! #### Authorship
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use std::ops::Not;

use crate::database::Schema;
use crate::model::solver::{IUtility, RUtility, SUtility};
use crate::solver::error::SolverError;
use crate::solver::{record, RecordType};

/* RECORD TYPE IMPLEMENTATIONS */

impl Into<String> for RecordType {
    fn into(self) -> String {
        match self {
            RecordType::MUR(players) => {
                format!("Real Utility Remoteness ({} players)", players)
            },
            RecordType::SUR(players) => {
                format!("Simple Utility Remoteness ({}  players)", players)
            },
            RecordType::REM => {
                format!("Remoteness (no utility)")
            },
        }
    }
}

impl TryInto<Schema> for RecordType {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Schema, Self::Error> {
        match self {
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
            v if v == SUtility::LOSE as i64 => Ok(SUtility::LOSE),
            v if v == SUtility::DRAW as i64 => Ok(SUtility::DRAW),
            v if v == SUtility::TIE as i64 => Ok(SUtility::TIE),
            v if v == SUtility::WIN as i64 => Ok(SUtility::WIN),
            _ => Err(SolverError::InvalidConversion {
                input_t: "Integer Utility".into(),
                output_t: "Simple Utility".into(),
                hint:
                    "Down-casting from integer to simple utility values is not \
                    stable, and relies on the internal representation used for \
                    simple utility values (which is not intuitive). As of \
                    right now though, WIN = 0, TIE = 3, DRAW = 2, and LOSE = 1."
                        .into(),
            }),
        }
    }
}

impl TryFrom<RUtility> for SUtility {
    type Error = SolverError;

    fn try_from(v: RUtility) -> Result<Self, Self::Error> {
        match v {
            v if v as i64 == SUtility::LOSE as i64 => Ok(SUtility::LOSE),
            v if v as i64 == SUtility::DRAW as i64 => Ok(SUtility::DRAW),
            v if v as i64 == SUtility::TIE as i64 => Ok(SUtility::TIE),
            v if v as i64 == SUtility::WIN as i64 => Ok(SUtility::WIN),
            _ => Err(SolverError::InvalidConversion {
                input_t: "Real Utility".into(),
                output_t: "Simple Utility".into(),
                hint:
                    "Simple Utility values can only have pre-specified values \
                    (which are subject to change)."
                        .into(),
            }),
        }
    }
}

impl TryFrom<u64> for SUtility {
    type Error = SolverError;

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            v if v as i64 == SUtility::LOSE as i64 => Ok(SUtility::LOSE),
            v if v as i64 == SUtility::DRAW as i64 => Ok(SUtility::DRAW),
            v if v as i64 == SUtility::TIE as i64 => Ok(SUtility::TIE),
            v if v as i64 == SUtility::WIN as i64 => Ok(SUtility::WIN),
            _ => Err(SolverError::InvalidConversion {
                input_t: "Real Utility".into(),
                output_t: "Simple Utility".into(),
                hint:
                    "Simple Utility values can only have pre-specified values \
                    (which are subject to change)."
                        .into(),
            }),
        }
    }
}

/* CONVERSIONS FROM SIMPLE UTILITY */

impl Into<IUtility> for SUtility {
    fn into(self) -> IUtility {
        match self {
            SUtility::LOSE => -1,
            SUtility::DRAW => 0,
            SUtility::TIE => 0,
            SUtility::WIN => 1,
        }
    }
}

impl Into<RUtility> for SUtility {
    fn into(self) -> RUtility {
        match self {
            SUtility::LOSE => -1.0,
            SUtility::DRAW => 0.0,
            SUtility::TIE => 0.0,
            SUtility::WIN => 1.0,
        }
    }
}

/* SIMPLE UTILITY NEGATION */

impl Not for SUtility {
    type Output = SUtility;
    fn not(self) -> Self::Output {
        match self {
            SUtility::DRAW => SUtility::DRAW,
            SUtility::LOSE => SUtility::WIN,
            SUtility::WIN => SUtility::LOSE,
            SUtility::TIE => SUtility::TIE,
        }
    }
}
