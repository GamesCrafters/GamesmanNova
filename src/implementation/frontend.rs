//! # Frontend Implementations
//!
//! TODO

use std::fmt;

use crate::model::frontend::IOMode;
use crate::model::frontend::InfoFormat;

/* UTILITY IMPLEMENTATIONS */

impl fmt::Display for IOMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IOMode::Constructive => write!(f, "constructive"),
            IOMode::Overwrite => write!(f, "overwrite"),
            IOMode::Forgetful => write!(f, "forgetful"),
        }
    }
}

impl fmt::Display for InfoFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfoFormat::Legible => write!(f, "legible"),
            InfoFormat::Json => write!(f, "json"),
        }
    }
}
