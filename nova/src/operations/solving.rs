//! # Solving Execution Module
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
    quiet: bool,
) -> Result<Value, UserError> {
    if !crate::games::LIST.contains(&&target[0..]) {
        Err(UserError::GameNotFoundError(target.to_owned()))
    } else {
        let target = &target[0..];
        let session = get_session::generate_match!("src/games/")(variant.to_owned());
        let solver_fn = find_solver(&session, solver.clone(), quiet)?;
        Ok(solver_fn(&session, read, write))
    }
}

/* HELPER FUNCTIONS */

fn find_solver<G: Solvable>(
    session: &G,
    solver: Option<String>,
    quiet: bool,
) -> Result<fn(&G, bool, bool) -> Value, UserError> {
    let available = session.solvers();
    if available.len() == 0 {
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
            if !quiet {
                println!("No solvers implemented for requested game.");
            }
            process::exit(exitcode::SOFTWARE);
        }
    }
}
