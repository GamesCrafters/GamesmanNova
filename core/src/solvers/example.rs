//! # Example Solver Module
//!
//! This module is not an actual solver, it is just a template for implementing
//! new solvers. The steps are outlined below, and step markers are placed 
//! throughout the template code to help you see where changes are needed.
//! 
//! STEP 0 - Do research and identify a game characteristic that makes it 
//!          possible to solve games in a special way, which we will call 
//!          <characteristic>.
//!
//! STEP 1 - Go to `core/src/solvers/mod.rs` and add a solving marker trait as 
//!          follows:
//! 
//! ```rust
//! // Add this below the existing ones
//! pub trait <characteristic>allySolvable
//! where
//!     Self: Game,
//!     Self: Solvable,
//! {
//!     // Please include any functions your solver will need to communicate
//!     // with a game implementation (beyond what is provided by the Game
//!     // trait) here.
//! }
//! ```
//!
//! STEP 2 - In the same file, add a public module declaration as follows:
//! 
//! ```rust
//! // Add this below the existing ones
//! pub mod <characteristic>;
//! ```
//!
//! STEP 3 - In this file, import the marker trait you made in Step 1.
//!
//! STEP 4 - Make this solver's name <characteristic>. Make sure the name is 
//!          not the same as any other solver in this folder.
//!
//! STEP 5 - Define a trait with a solve method and a solver_name method as 
//!          shown below.
//!
//! STEP 6 - Perform a blanket implementation for all <characteristic>lySolvable
//!          games of the trait you made in Step 5. Here, you are essentially 
//!          saying "every game which implements the trait from Step 1 will 
//!          automatically get the following implementation of the two methods
//!          outlined by the trait from Step 5." 
//!
//! STEP 7 - Write your solving algorithm in the solve method within the 
//!          implementation you created in Step 6. Make sure you supply the
//!          `read` and `write` arguments to a database implementation (see
//!          another implemented solver for an example).
//! 
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

// use super::<characteristic>   <-- STEP 3
use crate::Value;

/* SOLVER NAME */

/// Defines this solver's name for GamesmanNova's interfaces.
const SOLVER_NAME: &str = "<characteristic>";  // <-- STEP 4

/* COMFORTER IMPLEMENTATION */

// STEP 5 (uncomment the code block below)

// /// Indicates that a game could theoretically be solved <characteristic>ally.
// pub trait <characteristic>Solver {
//     /// Returns the value of an arbitrary state of the game, and uses `read` 
//     /// and `write` for specifying I/O preferences to database implementations.
//     fn <characteristic>_solve(game: &Self, read: bool, write: bool) -> Value;
//     /// Returns the name of this solver type.
//     fn <characteristic>_solver_name() -> String;
// }

// STEP 6 (uncomment the code block below)

// /// Blanket implementation of the <characteristic> solver for all <characteristic>ally
// /// solvable games.
// impl<G: <characteristic>allySolvable> <characteristic>Solver for G {
//     fn <characteristic>_solve(game: &Self, read: bool, write: bool) -> Value {
//         todo!()  <-- STEP 7
//     }
//
//     fn <characteristic>_solver_name() -> String {
//         SOLVER_NAME.to_owned()
//     }
// }
