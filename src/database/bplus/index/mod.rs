//! # Bplus Tree Module [WIP]
//!
//! This module provides a b+ tree data structure
//! that supports insert, delete, and lookup operations.

/* UTILITY MODULES */

mod error;

/* IMPLEMENTATION MODULES */

mod page;
mod page_manager;
mod tree;

/* IMPORTS */

pub use tree::{BTree, BTreeBuilder};

/* UNIT TESTING */

#[cfg(test)]
mod tests {
    use super::*;
}
