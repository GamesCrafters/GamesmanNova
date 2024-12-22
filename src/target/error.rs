//! # Extraction Target Error Module
//!
//! This module defines possible errors that could happen as a result of user
//! input or an incomplete extraction target implementation.

use std::{error::Error, fmt};

/* ERROR WRAPPER */

/// Wrapper for all target-related errors that could happen during runtime. Note
/// that the elements of this enumeration are all related to the implementation
/// of interface elements in `crate::target::mod`.
#[derive(Debug)]
pub enum TargetError {
    /// An error to indicate that a user attempted to extract a feature from a
    /// target variant which is valid, but offers no extractor for the specified
    /// feature.
    ExtractorNotFound {
        input_target_name: &'static str,
        feature_name: String,
    },

    /// An error to indicate that the variant passed to the target with
    /// `target_name` was not in a format the target could parse. Includes a
    /// message from the target implementation on exactly what went wrong. Note
    /// that `target_name` should be a valid argument to the `--target`
    /// parameter in the CLI.
    VariantMalformed {
        target_name: &'static str,
        hint: String,
    },

    /// An error to indicate that the state string passed to the target with
    /// `target_name` was not in a format the target could parse. Includes a
    /// message from the target implementation on exactly what went wrong. Note
    /// that `target_name` should be a valid argument to the `--target`
    /// parameter in the CLI.
    StateMalformed {
        target_name: &'static str,
        hint: String,
    },

    /// An error to indicate that a sequence of states in string form would
    /// be impossible to reproduce in real play. Includes a message from the
    /// target implementation on exactly what went wrong. Note: `target_name`
    /// should be a valid argument to the `--target` parameter in the CLI.
    InvalidHistory {
        target_name: &'static str,
        hint: String,
    },
}

impl Error for TargetError {}

impl fmt::Display for TargetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ExtractorNotFound {
                input_target_name,
                feature_name,
            } => {
                write!(
                    f,
                    "There is no extractor for the feature '{feature_name}' \
                    implemented for the target '{input_target_name}'."
                )
            },
            Self::VariantMalformed { target_name, hint } => {
                write!(
                    f,
                    "{hint}\n\nMore information on how the target expects you \
                    to format variant encodings can be found with 'nova info \
                    {target_name}'.",
                )
            },
            Self::StateMalformed { target_name, hint } => {
                write!(
                    f,
                    "{hint}\n\nMore information on how the target expects you \
                    to format state encodings can be found with 'nova info \
                    {target_name}'.",
                )
            },
            Self::InvalidHistory { target_name, hint } => {
                write!(
                    f,
                    "{hint}\n\nMore information on the target's rules can be \
                    found with 'nova info {target_name}'.",
                )
            },
        }
    }
}
