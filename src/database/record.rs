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
    models::{DrawDepth, MinimumExcludedValue, Player, Remoteness, Utility},
};
use colored::Colorize;
use nalgebra::SVector;
use std::fmt::Display;

/* DEFINITION */

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct Record<const N: usize> {
    pub util: SVector<Utility, N>,
    pub draw: DrawDepth,
    pub rem: Remoteness,
    pub mex: MinimumExcludedValue,
}

/* IMPLEMENTATION */

impl<const N: usize> Record<N> {
    /* RECORD BUILDER LITE (TM) */

    pub fn with_util(mut self, util: SVector<Utility, N>) -> Self {
        self.util = util;
        self
    }

    pub fn with_draw(mut self, draw: DrawDepth) -> Self {
        self.draw = draw;
        self
    }

    pub fn with_rem(mut self, rem: Remoteness) -> Self {
        self.rem = rem;
        self
    }

    pub fn with_mex(mut self, mex: MinimumExcludedValue) -> Self {
        self.mex = mex;
        self
    }

    /* RECORD UTILS */

    pub fn get_utility(&self, player: Player) -> Utility {
        if let Some(utility) = self
            .util
            .get(player as usize)
        {
            *utility
        } else {
            panic!(
                "Out-of-bounds vector access: Attempted to fetch utility for \
                player {} in a game of {} players.",
                player,
                self.util.nrows()
            )
        }
    }

    pub fn format(&self, format: Option<OutputFormat>) -> Option<String> {
        match format {
            Some(OutputFormat::None) => None,
            Some(OutputFormat::Extra) => {
                todo!()
            },
            Some(OutputFormat::Json) => Some(
                serde_json::json!({
                        "utility": *self.util.to_string(),
                        "remoteness": self.rem,
                        "draw_depth": self.draw,
                        "mex": self.mex,
                })
                .to_string(),
            ),
            None => Some(format!(
                "{} {}\n{} {}\n{} {}\n{} {}",
                "Utility:"
                    .green()
                    .bold(),
                self.util,
                "Remoteness:".bold(),
                self.rem,
                "Draw depth:".bold(),
                self.draw,
                "Mex:".bold(),
                self.mex,
            )),
        }
    }
}

/* STANDARD IMPLEMENTATIONS */

impl<const N: usize> Default for Record<N> {
    fn default() -> Self {
        Record {
            util: SVector::<Utility, N>::zeros(),
            draw: u64::default(),
            rem: u64::default(),
            mex: u64::default(),
        }
    }
}

impl<const N: usize> Display for Record<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n{}{} {}\n{} {}\n{} {}",
            "Utility vector:"
                .green()
                .bold(),
            self.util,
            "Remoteness:".bold(),
            self.rem,
            "Draw depth:".bold(),
            self.draw,
            "Mex:".bold(),
            self.mex,
        )
    }
}
