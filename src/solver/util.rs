//! # Solver Utilities Module
//!
//! This module makes room for common utility routines used throughout the
//! `crate::solver` module.
//!
//! #### Authorship
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use crate::database::Schema;
use crate::model::solver::{IUtility, SUtility};
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

/* UTILITY CONVERSION */

impl TryFrom<IUtility> for SUtility {
    type Error = SolverError;

    fn try_from(v: IUtility) -> Result<Self, Self::Error> {
        match v {
            v if v == SUtility::LOSE as i64 => Ok(SUtility::LOSE),
            v if v == SUtility::DRAW as i64 => Ok(SUtility::DRAW),
            v if v == SUtility::TIE as i64 => Ok(SUtility::TIE),
            v if v == SUtility::WIN as i64 => Ok(SUtility::WIN),
            _ => Err(todo!()),
        }
    }
}

impl TryFrom<u64> for SUtility {
    type Error = SolverError;

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            v if v == SUtility::LOSE as u64 => Ok(SUtility::LOSE),
            v if v == SUtility::DRAW as u64 => Ok(SUtility::DRAW),
            v if v == SUtility::TIE as u64 => Ok(SUtility::TIE),
            v if v == SUtility::WIN as u64 => Ok(SUtility::WIN),
            _ => Err(todo!()),
        }
    }
}

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
