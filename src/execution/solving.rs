//! # Solving Execution Module
//!
//! This module contains handling behavior for all `nova solve` requests.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::errors::NotFoundError;
use crate::games::Game;
use crate::interfaces::find_game;
use crate::interfaces::terminal::cli::SolveArgs;
use crate::models::Solver;
use std::error::Error;

/// Attempts to solve the game with the indicated `name`, and returns the value
/// or an error containing what was actually passed in versus what was
/// probably meant to be passed in.
pub fn solve_by_name(args: &SolveArgs) -> Result<(), Box<dyn Error>>
{
    let game = find_game(args.target, args.variant.clone())?;
    let solver = find_solver(&game, args.solver.clone())?;
    solver(&game, args.mode);
    Ok(())
}

/* HELPER FUNCTIONS */

/// Probes the provided game for a solver with the indicated name if one is
/// provided. If no solver name is provided, returns any one of the solvers
/// which the game returns.If no name match is found or if there are no solvers
/// available, an error is provided to the user with a suggestion.
fn find_solver<G: Game>(
    game: &G,
    solver: Option<String>,
) -> Result<Solver<G>, NotFoundError>
{
    let solvers = game.solvers();
    if let Some(solver_name) = solver {
        if let Some(solver) = solvers.get(&solver_name) {
            Ok(*solver)
        } else {
            Err(NotFoundError::Solver {
                solver_name,
                game_name: game.info().name,
                available_solvers: solvers
                    .keys()
                    .into_iter()
                    .map(|s| s.clone())
                    .collect(),
            })
        }
    } else {
        Ok(*solvers.values().next().unwrap())
    }
}
