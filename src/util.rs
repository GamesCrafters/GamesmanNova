//! # Utilities Module
//!
//! This module makes room for verbose or repeated routines used in the
//! top-level module of this crate.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};
use clap::ValueEnum;
use serde_json::json;

use std::{fmt::Display, process};

use crate::{
    game::{zero_by, Game, GameData},
    interface::{IOMode, OutputMode},
};

/* DATA STRUCTURES */

// Specifies the game offerings available through all interfaces.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum GameModule {
    ZeroBy,
}

/* SUBROUTINES */

/// Fetches and initializes the correct game session based on an indicated
/// `GameModule`, with the provided `variant`.
pub fn find_game(
    game: GameModule,
    variant: Option<String>,
    state: Option<String>,
) -> Result<Box<dyn Game>> {
    match game {
        GameModule::ZeroBy => {
            Ok(Box::new(zero_by::Session::initialize(variant)?))
        },
    }
}

/// Prompts the user to confirm their operation as appropriate according to
/// the arguments of the solve command. Only asks for confirmation for
/// potentially destructive operations.
pub fn confirm_potential_overwrite(yes: bool, mode: IOMode) {
    if match mode {
        IOMode::Write => !yes,
        IOMode::Find => false,
    } {
        println!(
            "This may overwrite an existing solution database. Are you sure? \
            [y/n]: "
        );
        let mut yn: String = "".to_owned();
        while !["n", "N", "y", "Y"].contains(&&yn[..]) {
            yn = String::new();
            std::io::stdin()
                .read_line(&mut yn)
                .expect("Failed to read user confirmation.");
            yn = yn.trim().to_string();
        }
        if yn == "n" || yn == "N" {
            process::exit(exitcode::OK)
        }
    }
}

/// Prints the formatted game information according to a specified output
/// format. Game information is provided by game implementations.
pub fn print_game_info(game: GameModule, format: OutputMode) -> Result<()> {
    find_game(game, None, None)
        .context("Failed to initialize game session.")?
        .info()
        .print(format);
    Ok(())
}

/* IMPLEMENTATIONS */

impl GameData<'_> {
    fn print(&self, format: OutputMode) {
        match format {
            OutputMode::Extra => {
                let content = format!(
                    "\tGame:\n{}\n\n\tAuthor:\n{}\n\n\tDescription:\n{}\n\n\t\
                    Variant Protocol:\n{}\n\n\tVariant Default:\n{}\n\n\t\
                    Variant Pattern:\n{}\n\n\tState Protocol:\n{}\n\n\tState \
                    Default:\n{}\n\n\tState Pattern:\n{}\n",
                    self.name,
                    self.authors,
                    self.about,
                    self.variant_protocol,
                    self.variant_default,
                    self.variant_pattern,
                    self.state_protocol,
                    self.state_default,
                    self.state_pattern
                );
                println!("{}", content);
            },
            OutputMode::Json => {
                let content = json!({
                    "game": self.name,
                    "author": self.authors,
                    "about": self.about,
                    "variant-protocol": self.variant_protocol,
                    "variant-default": self.variant_default,
                    "variant-pattern": self.variant_pattern,
                    "state-protocol": self.state_protocol,
                    "state-default": self.state_default,
                    "state-pattern": self.state_pattern,
                });
                println!("{}", content);
            },
            OutputMode::None => (),
        }
    }
}

impl Display for GameData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\tGame:\n{}\n\n\tAuthor:\n{}\n\n\tDescription:\n{}\n\n\tVariant \
            Protocol:\n{}\n\n\tVariant Default:\n{}\n\n\tVariant Pattern:\n{}\
            \n\n\tState Protocol:\n{}\n\n\tState Default:\n{}\n\n\tState \
            Pattern:\n{}\n",
            self.name,
            self.authors,
            self.about,
            self.variant_protocol,
            self.variant_default,
            self.variant_pattern,
            self.state_protocol,
            self.state_default,
            self.state_pattern
        )
    }
}

/* DECLARATIVE MACROS */

/// Syntax sugar. Implements multiple traits for a single concrete type. The
/// traits implemented must be marker traits; in other words, they must have no
/// behavior (no functions). You will usually want to use this for implementing
/// all the solvers for a game ergonomically through their marker traits.
///
/// Example usage:
///
/// ```no_run
/// implement! { for Game =>
///     AcyclicGame,
///     AcyclicallySolvable,
///     TreeSolvable,
///     TierSolvable
/// }
/// ```
///
/// ...which expands to the following:
///
/// ```no_run
/// impl AcyclicallySolvable for Game {}
///
/// impl TreeSolvable for Game {}
///
/// impl TierSolvable for Game {}
/// ```
#[macro_export]
macro_rules! implement {
    (for $b:ty => $($t:ty),+) => {
        $(impl $t for $b { })*
    }
}

/// Syntax sugar. Allows a "literal-like" declaration of collections like
/// `HashSet`s, `HashMap`s, `Vec`s, etc.
///
/// Example usage:
///
/// ```no_run
/// let s: Vec<_> = collection![1, 2, 3];
/// let s: HashSet<_> = collection! { 1, 2, 3 };
/// let s: HashMap<_, _> = collection! { 1 => 2, 3 => 4 };
/// ```
/// ...which expands to the following:
///
/// ```no_run
/// let s = Vec::from([1, 2, 3]);
/// let s = HashSet::from([1, 2, 3]);
/// let s = HashMap::from([(1, 2), (3, 4)]);
/// ```
#[macro_export]
macro_rules! collection {
    ($($k:expr => $v:expr),* $(,)?) => {{
        core::convert::From::from([$(($k, $v),)*])
    }};
    ($($v:expr),* $(,)?) => {{
        core::convert::From::from([$($v,)*])
    }};
}

/// Syntax sugar. Allows for a declarative way of expressing attribute names
/// and sizes for constructing database schemas.
///
/// Example usage:
///
/// ```no_run
/// let s1 = schema!("attribute1": 10, "attribute2": 5);
/// let s2 = schema! {
///     "attribute3": 20,
///     "attribute4": 60,   
/// };
/// ```
///
/// ...which expands to the following:
///
/// ```no_run
/// let s1 = SchemaBuilder::new()
///     .add(Attribute::new("attribute1", 10))?
///     .add(Attribute::new("attribute2", 5))?
///     .build();
///
/// let s2 = SchemaBuilder::new()
///     .add(Attribute::new("attribute3", 20))?
///     .add(Attribute::new("attribute4", 60))?
///     .build();
/// ```
#[macro_export]
macro_rules! schema {
    {$($key:literal: $value:expr),*} => {
        SchemaBuilder::new()
            $(
                .add(Attribute::new($key, $value))?
            )*
            .build()
    };
}
