//! # User Errors Module
//!
//! This module defines the errors that can happen as a result of user input.
//! These errors are not for malformed input, but rather things that happen
//! as a result of the user not knowing the offerings of the program.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use std::{error::Error, fmt};

/// An error to indicate that a user input the name of a game which is not
/// included as a target. Supports telling the user what they typed and
/// a suggestion, presumably using a string distance calculator.
#[derive(Debug)]
pub struct GameNotFoundError {
    pub input: String,
    pub suggestion: String,
}

impl Error for GameNotFoundError {}

impl fmt::Display for GameNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The game '{}' was not found among the offerings. Perhaps you meant '{}'?",
            self.input, self.suggestion
        )
    }
}
