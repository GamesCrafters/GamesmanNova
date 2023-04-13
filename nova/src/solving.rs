//! # Nova Solving Module
//!
//! This module contains handling behavior for all `nova solve ...` requests.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::errors::user::UserError;
use core::{archetypes::Game, Value};
use std::process;

/// Attempts to solve the game with the indicated `name`, and returns the value
/// or an error containing what was actually passed in versus what was
/// probably meant to be passed in.
pub fn solve_by_names(game: &str, solver: Option<String>) -> Result<Value, UserError> {
    match game {
        "10-to-0-by-1-or-2" => {
            let session = games::ten_to_zero_by_one_or_two::Session::new();
            let found_solver = find_solver(&session, solver)?;
            Ok(found_solver(&session))
        }
        _ => {
            let not_found_error = UserError::GameNotFoundError(game.to_owned());
            Err(not_found_error)
        }
    }
}

fn find_solver<G: Game>(session: &G, solver: Option<String>) -> Result<fn(&G) -> Value, UserError> {
    let available = session.solvers();
    if available.len() == 0 {
        println!("No solvers implemented for requested game.");
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
        return Err(UserError::SolverNotFoundError(target, names));
    } else {
        for (solver_name, solver_func) in available {
            if let None = solver_name {
                return Ok(solver_func.to_owned());
            }
        }
        if let Some((_, solver_func)) = session.solvers().get(0) {
            return Ok(solver_func.to_owned());
        } else {
            println!("No solvers implemented for requested game.");
            process::exit(exitcode::SOFTWARE);
        }
    }
}
