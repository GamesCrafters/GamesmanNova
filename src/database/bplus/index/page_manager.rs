//! # Bplus Tree Page Manager [WIP]
//!
//! This file contains a page manager to handle
//! page fetches and writes for a b+ tree instance.

/* IMPORTS */

use crate::database::bplus::index::error::Error;
use anyhow::Result;

/* DEFINITIONS */

pub struct PageManager {}

/* IMPLEMENTATIONS */

impl PageManager {
    pub fn new() -> Result<PageManager, Error> {
        todo!()
    }

    pub fn fetch_page() -> Result<(), Error> {
        todo!()
    }

    pub fn write_page() -> Result<(), Error> {
        todo!()
    }
}

/* UNIT TESTING */

#[cfg(test)]
mod tests {
    use super::*;
}
