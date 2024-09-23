//! # Bplus Tree Page [WIP]
//!
//! This file contains a page wrapper for system
//! memory with functions to mutate page data.

/* IMPORTS */

use crate::database::bplus::index::error::Error;
use anyhow::Result;

/* CONSTANTS */

const PAGE_SIZE: usize = 4096;

/* DEFINITIONS */

pub struct Byte(u8);

pub struct Page {
    data: Box<[Byte; PAGE_SIZE]>,
}

/* IMPLEMENTATIONS */

impl Page {
    pub fn new(data: Box<[Byte; PAGE_SIZE]>) {
        todo!()
    }

    pub fn read_at_offset(
        &self,
        offset: usize,
        size: usize,
    ) -> Result<&[Byte], Error> {
        todo!()
    }

    pub fn write_at_offset(
        &self,
        offset: usize,
        contents: &[Byte],
    ) -> Result<(), Error> {
        todo!()
    }
}

/* UNIT TESTING */

#[cfg(test)]
mod tests {
    use super::*;
}
