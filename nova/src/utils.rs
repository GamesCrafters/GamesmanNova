//! # Utilities Module
//!
//! This module factors out common behavior across this project.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use crate::{
    interfaces::terminal::cli::{IOMode, OutputFormat},
    models::Record,
};
use colored::Colorize;
use std::process;

/* ALGORITHMS */

/// Returns the most similar string to `model` in the vector `all`. Used for
/// checking user input against offerings to provide useful suggestions for
/// malformed command arguments.
pub fn most_similar(model: &str, all: Vec<&str>) -> String
{
    let mut best = usize::MAX;
    let mut closest = "";
    let mut current;
    for s in all {
        current = strsim::damerau_levenshtein(model, s);
        if current <= best {
            closest = s;
            best = current;
        }
    }
    closest.to_owned()
}

/* PRINTING AND OTHER I/O */

/// Prompts the user to confirm their operation as appropriate according to
/// the arguments of the solve command. Only asks for confirmation for
/// potentially destructive operations.
pub fn confirm_potential_overwrite(yes: bool, mode: Option<IOMode>)
{
    if match mode {
        Some(IOMode::Write) => !yes,
        _ => false,
    } {
        println!("This may overwrite an existing solution database. Are you sure? [y/n]: ");
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

/// Returns a `record` represented in a specific `format`.
pub fn format_record<const N: usize>(
    record: &Record<N>,
    format: Option<OutputFormat>,
) -> Option<String>
{
    match format {
        Some(OutputFormat::None) => None,
        Some(OutputFormat::Extra) => {
            todo!()
        }
        Some(OutputFormat::Json) => Some(
            serde_json::json!({
                    "utility": record.utility.to_string(),
                    "remoteness": record.remoteness,
                    "draw_depth": record.draw_depth,
                    "mex": record.mex,
            })
            .to_string(),
        ),
        None => Some(format!(
            "{} {}\n{} {}\n{} {}\n{} {}",
            "Utility:".green().bold(),
            record.utility,
            "Remoteness:".bold(),
            record.remoteness,
            "Draw depth:".bold(),
            record.draw_depth,
            "Mex:".bold(),
            record.mex,
        )),
    }
}

/* MACROS */

/// Implements multiple traits for a single concrete type. The traits
/// implemented must be marker traits; in other words, they must have no
/// behavior (no functions). You will usually want to use this for implementing
/// all the solvers for a game ergonomically through their marker traits.
///
/// Example usage:
///
/// ```none
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
/// ```none
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
