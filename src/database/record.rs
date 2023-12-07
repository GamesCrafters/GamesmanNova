#![allow(dead_code, unused_variables)]
//! # Database Record Definition Module
//!
//! This module implements a static-size record storing the attributes
//! associated with a game position. Since it is usually unfeasible to store
//! game state hashes as primary keys, it is assumed that the DBMS which uses
//! this definition of a record handles fetching and storing through some method
//! other than storing primary keys in records.
//!
//! #### Authorship
//!
//! - Max Fierro, 11/4/2023 (maxfierro@berkeley.edu)

use crate::{
    interfaces::terminal::cli::OutputFormat,
    models::{
        DrawDepth, MinimumExcludedValue, PlayerCount, Remoteness, Turn, Utility,
    },
};
use colored::Colorize;
use nalgebra::SVector;
use std::{cmp::Ordering, fmt::Display};

/* DEFINITION */

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct Record<const N: PlayerCount> {
    pub utility: SVector<Utility, N>,
    pub remoteness: Remoteness,
    pub draw: DrawDepth,
    pub mex: MinimumExcludedValue,
}

/* IMPLEMENTATION */

impl<const N: PlayerCount> Record<N> {
    /* RECORD BUILDER LITE (TM) */

    /// Mutates `self` to have `utility` and returns it.
    pub fn with_utility(mut self, utility: SVector<Utility, N>) -> Self {
        self.utility = utility;
        self
    }

    /// Mutates `self` to have `draw` and returns it.
    pub fn with_draw(mut self, draw: DrawDepth) -> Self {
        self.draw = draw;
        self
    }

    /// Mutates `self` to have `remoteness` and returns it.
    pub fn with_remoteness(mut self, remoteness: Remoteness) -> Self {
        self.remoteness = remoteness;
        self
    }

    /// Mutates `self` to have `mex` and returns it.
    pub fn with_mex(mut self, mex: MinimumExcludedValue) -> Self {
        self.mex = mex;
        self
    }

    /* RECORD UTILS */

    /// Returns whether the utility associated with `player` in this record is
    /// greater, less than, or equal to the utility associated with the same
    /// player in `other`. When utility is equal, potential ties are broken with
    /// remoteness if possible.
    ///
    /// **WARNING:** Assumes that `player` is valid for the player count of this
    /// record (which is specified by its generic parameter `N`) for the small
    /// performance benefit of an unchecked array access.
    pub fn cmp(&self, other: &Record<N>, player: Turn) -> Ordering {
        let u1 = self.utility[player];
        let u2 = other.utility[player];
        if u1 != u2 {
            u1.cmp(&u2)
        } else {
            self.remoteness
                .cmp(&other.remoteness)
        }
    }

    /// Returns the utility associated with `player` in this record, providing
    /// a useful panic message if the player identifier is larger than expected
    /// for a game of size `N`, where `N` is this record's generic parameter.
    pub fn get_utility(&self, player: Turn) -> Utility {
        if let Some(utility) = self.utility.get(player) {
            *utility
        } else {
            panic!(
                "Out-of-bounds vector access: Attempted to fetch utility for \
                player {} in a game of {} players.",
                player,
                self.utility.nrows()
            )
        }
    }

    /// Returns information about this record as a string according to the
    /// specified `format`. Note that there is not necessarily a way to know
    /// which values are left unused in the record.
    pub fn format(&self, format: Option<OutputFormat>) -> Option<String> {
        match format {
            Some(OutputFormat::None) => None,
            Some(OutputFormat::Extra) => {
                todo!()
            },
            Some(OutputFormat::Json) => Some(
                serde_json::json!({
                        "utility": *self.utility.to_string(),
                        "remoteness": self.remoteness,
                        "draw_depth": self.draw,
                        "mex": self.mex,
                })
                .to_string(),
            ),
            None => Some(format!(
                "{} {}\n{} {}\n{} {}\n{} {}",
                "Utility:".green().bold(),
                self.utility,
                "Remoteness:".bold(),
                self.remoteness,
                "Draw depth:".bold(),
                self.draw,
                "Mex:".bold(),
                self.mex,
            )),
        }
    }
}

/* STANDARD IMPLEMENTATIONS */

impl<const N: PlayerCount> Default for Record<N> {
    fn default() -> Self {
        Record {
            utility: SVector::<Utility, N>::zeros(),
            draw: u64::default(),
            remoteness: u64::default(),
            mex: u64::default(),
        }
    }
}

impl<const N: PlayerCount> Display for Record<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n{}{} {}\n{} {}\n{} {}",
            "Utility vector:".green().bold(),
            self.utility,
            "Remoteness:".bold(),
            self.remoteness,
            "Draw depth:".bold(),
            self.draw,
            "Mex:".bold(),
            self.mex,
        )
    }
}
