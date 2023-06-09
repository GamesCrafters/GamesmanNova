//! # Solving Execution Module
//!
//! This module contains handling behavior for all `nova solve` requests.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::core::solvers::Solvable;
use crate::core::{Solver, Value};
use crate::errors::NovaError;
use crate::games::Game;
use crate::interfaces::terminal::cli::{Output, SolveArgs};
use crate::utils::check_game_exists;
use serde_json::json;
use std::process;

/// Attempts to solve the game with the indicated `name`, and returns the value
/// or an error containing what was actually passed in versus what was
/// probably meant to be passed in.
pub fn solve_by_name(
    target: &String,
    variant: &Option<String>,
    solver: &Option<String>,
    read: bool,
    write: bool,
    quiet: bool,
) -> Result<Value, NovaError> {
    check_game_exists(target)?;
    let target: &str = target;
    let session = get_session::generate_match!("src/games/")(variant.to_owned());
    let solver_fn = find_solver(&session, solver.clone(), quiet)?;
    Ok(solver_fn(&session, read, write))
}

/// Prints the result of a solve on a particular game in the specified format,
/// if any.
pub fn printf_solve_result(value: Value, args: &SolveArgs) {
    let value_str: &str;
    let remoteness: u32;
    match value {
        Value::Lose(rem) => {
            value_str = "lose";
            remoteness = rem;
        }
        Value::Tie(rem) => {
            value_str = "tie";
            remoteness = rem;
        }
        Value::Win(rem) => {
            value_str = "win";
            remoteness = rem;
        }
    }
    if let Some(format) = args.output {
        match format {
            Output::Extra => {
                println!(
                    "\nYou solved {}. The game is a {} for the first player in {} moves.\n",
                    args.target, value_str, remoteness
                );
            }
            Output::Json => {
                let json = json!({
                    "value": value_str,
                    "remoteness": remoteness
                });
                println!("{}", json);
            }
        }
    } else {
        println!("{} in {}", value_str, remoteness);
    }
}

/// Prompts the user to confirm their operation as appropriate according to
/// the arguments of the solve command. Only asks for confirmation for
/// potentially destructive operations.
pub fn confirm_potential_overwrite(args: &SolveArgs) {
    if (!args.yes) && args.write {
        println!("This may overwrite an existing solution database. Are you sure? [y/n]: ");
        let mut yn: String = "".to_owned();
        while !(yn == "n" || yn == "N" || yn == "y" || yn == "Y") {
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

/* HELPER FUNCTIONS */

fn find_solver<G: Solvable>(
    session: &G,
    solver: Option<String>,
    quiet: bool,
) -> Result<Solver<G>, NovaError> {
    let available = session.solvers();
    if available.is_empty() {
        if !quiet {
            println!("No solvers implemented for requested game.");
        }
        process::exit(exitcode::SOFTWARE);
    }
    if let Some(target) = solver {
        let mut names = vec![];
        for (solver_name, solver_func) in available {
            if let Some(candidate) = solver_name {
                if candidate == target {
                    return Ok(solver_func.to_owned());
                }
                names.push(candidate.clone().to_owned());
            }
        }
        Err(NovaError::SolverNotFoundError(target, names))
    } else {
        for (solver_name, solver_func) in available {
            if solver_name.is_none() {
                return Ok(solver_func.to_owned());
            }
        }
        if let Some((_, solver_func)) = session.solvers().get(0) {
            Ok(solver_func.to_owned())
        } else {
            if !quiet {
                println!("No solvers implemented for requested game.");
            }
            process::exit(exitcode::SOFTWARE);
        }
    }
}
