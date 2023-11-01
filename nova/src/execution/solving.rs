//! # Solving Execution Module
//!
//! This module contains handling behavior for all `nova solve` requests.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::games::Solvable;
use crate::interfaces::find_game;
use crate::interfaces::terminal::cli::SolveArgs;
use crate::models::{Record, Solver};
use crate::utils::most_similar;
use std::collections::HashMap;
use std::process;

/// Attempts to solve the game with the indicated `name`, and returns the value
/// or an error containing what was actually passed in versus what was
/// probably meant to be passed in.
pub fn solve_by_name<const N: usize>(args: &SolveArgs, quiet: bool)
    -> Record<N>
{
    let game = find_game(args.target, args.variant);
    let solver = find_solver(&game, args.solver.clone(), quiet);
    solver(&game, args.mode)
}

/* HELPER FUNCTIONS */

/// Probes the provided game for a solver with the indicated name if one is
/// provided. If no solver name is provided, returns any one of the solvers
/// which the game returns.If no name match is found or if there are no solvers
/// available, an error is provided to the user with a suggestion.
fn find_solver<G: Solvable<N>, const N: usize>(
    game: &G,
    solver: Option<String>,
    quiet: bool,
) -> Solver<G, N>
{
    let solvers: HashMap<&str, Solver<G>> = game.solvers();
    if solvers.is_empty() {
        if !quiet {
            println!("There are no solvers associated with this game.");
        }
        process::exit(exitcode::SOFTWARE);
    } else if let Some(solver_name) = solver {
        *solvers.get(&solver_name[..]).unwrap_or_else(|| {
            if !quiet {
                let closest = most_similar(
                    &solver_name,
                    Vec::from_iter(solvers.keys())
                        .iter()
                        .map(|x| **x)
                        .collect(),
                );
                println!(
                    "There is no solver named {}. Perhaps you meant: {}",
                    solver_name, closest
                );
            }
            process::exit(exitcode::USAGE);
        })
    } else {
        *solvers.values().next().unwrap()
    }
}
