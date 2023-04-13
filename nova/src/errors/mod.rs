//! # General Errors Module
//!
//! This is the root module for the different types of errors that can occur
//! during Nova's execution.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use std::error::Error;

pub mod implementation;
pub mod user;

pub trait NovaError
where
    Self: Error,
{
}
