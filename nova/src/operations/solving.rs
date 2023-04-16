//! # Nova Solving Module
//!
//! This module contains handling behavior for all `nova solve` requests.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::core::solvers::Solvable;
use crate::core::Value;
use crate::errors::UserError;
use crate::games::Game;
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
) -> Result<Value, UserError> {
    match &target[0..] {
        "zero-by" => {
            let session = crate::games::zero_by::Session::initialize(variant.clone());
            let found_solver = find_solver(&session, solver.clone())?;
            Ok(found_solver(&session, read, write))
        }
        _ => {
            let not_found_error = UserError::GameNotFoundError(target.to_owned());
            Err(not_found_error)
        }
    }
}

/* HELPER FUNCTIONS */

fn find_solver<G>(
    session: &G,
    solver: Option<String>,
) -> Result<fn(&G, bool, bool) -> Value, UserError>
where
    G: Solvable,
{
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
