//! # Example Game Module - General Utility General Sum [WIP]
//!
//! This module contains games of type tree, acyclic, and cyclic that
//! adhere to the following definitions:
//!     General Utility - Player utilities are expressed numerically.
//!     General Sum - Sum of all utilities is not necessarily zero.

/* IMPORTS */

use crate::game::mock::example::{
    AcyclicExampleGame, CyclicExampleGame, TreeExampleGame, Visualizer,
};
use crate::game::mock::{builder, Node};
use crate::node;
use anyhow::Result;

/* CONSTANTS */

const TREE_GAME_NAME: &str = "general-utility-general-sum-tree-structure";
const ACYCLIC_GAME_NAME: &str = "general-utility-general-sum-acyclic-structure";
const CYCLIC_GAME_NAME: &str = "general-utility-general-sum-cyclic-structure";

/* DEFINITIONS */

trait ExampleGame<'a>: Sized {
    fn new(nodes: &'a mut Vec<Node>) -> Result<Self>;
}

/* IMPLEMENTATIONS */

impl<'a> ExampleGame<'a> for TreeExampleGame<'a> {
    fn new(store: &'a mut Vec<Node>) -> Result<TreeExampleGame<'a>> {
        let mut nodes = vec![
            node!(0),
            node!(1),
            node!(1),
            node!(1),
            node!(0),
            node!(0),
            node!(0),
            node!(0),
            node!(0),
            node!(0),
            node![-1, 1],
            node![2, -3],
            node![2, 1],
            node![0, 0],
            node![4, 5],
            node![-1, -2],
            node![-2, 2],
            node![4, -5],
            node![-1, 0],
        ];

        let length = store.len();
        store.append(&mut nodes);
        let store = &store[length..];

        let game = builder::SessionBuilder::new(&TREE_GAME_NAME)
            .edge(&store[0], &store[1])?
            .edge(&store[0], &store[2])?
            .edge(&store[0], &store[3])?
            .edge(&store[1], &store[4])?
            .edge(&store[1], &store[5])?
            .edge(&store[1], &store[6])?
            .edge(&store[2], &store[7])?
            .edge(&store[2], &store[8])?
            .edge(&store[2], &store[9])?
            .edge(&store[3], &store[10])?
            .edge(&store[3], &store[11])?
            .edge(&store[3], &store[12])?
            .edge(&store[4], &store[13])?
            .edge(&store[5], &store[14])?
            .edge(&store[6], &store[15])?
            .edge(&store[7], &store[16])?
            .edge(&store[8], &store[17])?
            .edge(&store[9], &store[18])?
            .start(&store[0])?
            .build()?;

        Ok(TreeExampleGame { game })
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
