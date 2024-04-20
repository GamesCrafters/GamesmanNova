//! # Example Game Module - General Utility Zero Sum [WIP]
//!
//! This module contains games of type tree, acyclic, and cyclic that
//! adhere to the following definitions:
//!     General Utility - Player utilities are expressed numerically.
//!     Zero Sum - Sum of all utilities is zero.

/* IMPORTS */

use crate::game::mock::example::{
    AcyclicExampleGame, CyclicExampleGame, TreeExampleGame, Visualizer,
};
use crate::game::mock::{builder, Node};
use crate::node;
use anyhow::Result;

/* CONSTANTS */

const TREE_GAME_NAME: &str = "general-utility-zero-sum-tree-structure";
const ACYCLIC_GAME_NAME: &str = "general-utility-zero-sum-acyclic-structure";
const CYCLIC_GAME_NAME: &str = "general-utility-zero-sum-cyclic-structure";

/* DEFINITIONS */

trait ExampleGame<'a>: Sized {
    fn new(nodes: &'a mut Vec<Node>) -> Result<Self>;
}

/* IMPLEMENTATIONS */

impl<'a> ExampleGame<'a> for TreeExampleGame<'a> {
    fn new(store: &'a mut Vec<Node>) -> Result<TreeExampleGame<'a>> {
        todo!()
    }
}

impl<'a> ExampleGame<'a> for AcyclicExampleGame<'a> {
    fn new(store: &'a mut Vec<Node>) -> Result<AcyclicExampleGame<'a>> {
        todo!()
    }
}

impl<'a> ExampleGame<'a> for CyclicExampleGame<'a> {
    fn new(store: &'a mut Vec<Node>) -> Result<CyclicExampleGame<'a>> {
        todo!()
    }
}

/* TESTING */

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn initialize_example_tree_game() -> Result<()> {
        let _ = TreeExampleGame::new(&mut vec![])?;
        Ok(())
    }

    #[test]
    fn initialize_example_acyclic_game() -> Result<()> {
        let _ = AcyclicExampleGame::new(&mut vec![])?;
        Ok(())
    }

    #[test]
    fn initialize_example_cyclic_game() -> Result<()> {
        let _ = CyclicExampleGame::new(&mut vec![])?;
        Ok(())
    }

    #[test]
    fn visualize_example_tree_game() -> Result<()> {
        let _ = TreeExampleGame::new(&mut vec![])?.visualize();
        Ok(())
    }

    #[test]
    fn visualize_example_acyclic_game() -> Result<()> {
        let _ = AcyclicExampleGame::new(&mut vec![])?.visualize();
        Ok(())
    }

    #[test]
    fn visualize_example_cyclic_game() -> Result<()> {
        let _ = CyclicExampleGame::new(&mut vec![])?.visualize();
        Ok(())
    }
}
