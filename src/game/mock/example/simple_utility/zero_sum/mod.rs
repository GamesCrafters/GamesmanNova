//! # Example Game Module - Simple Utility Zero Sum
//!
//! This module contains games of type tree, acyclic, and cyclic that
//! adhere to the following definitions:
//!     Simple Utility - Player utilities are defined as WIN, LOSE, TIE, or DRAW.
//!     Zero Sum - Sum of all utilities is zero.

/* IMPORTS */

use crate::game::mock::example::{
    AcyclicExampleGame, CyclicExampleGame, TreeExampleGame, Visualizer,
};
use crate::game::mock::{builder, Node};
use crate::game::SimpleUtility;
use crate::node;
use anyhow::Result;

/* CONSTANTS */

const TREE_GAME_NAME: &str = "simple-utility-zero-sum-tree-structure";
const ACYCLIC_GAME_NAME: &str = "simple-utility-zero-sum-acyclic-structure";
const CYCLIC_GAME_NAME: &str = "simple-utility-zero-sum-cyclic-structure";

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
            node!(0),
            node!(0),
            node!(1),
            node!(1),
            node!(1),
            node!(1),
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::TIE.into(),
                SimpleUtility::TIE.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::TIE.into(),
                SimpleUtility::TIE.into(),
            ],
            node![
                SimpleUtility::TIE.into(),
                SimpleUtility::TIE.into(),
            ],
        ];

        let length = store.len();
        store.append(&mut nodes);
        let store = &store[length..];

        let game = builder::SessionBuilder::new(&TREE_GAME_NAME)
            .edge(&store[0], &store[1])?
            .edge(&store[0], &store[2])?
            .edge(&store[1], &store[3])?
            .edge(&store[1], &store[4])?
            .edge(&store[2], &store[9])?
            .edge(&store[2], &store[10])?
            .edge(&store[3], &store[5])?
            .edge(&store[3], &store[6])?
            .edge(&store[4], &store[7])?
            .edge(&store[4], &store[8])?
            .edge(&store[5], &store[11])?
            .edge(&store[5], &store[12])?
            .edge(&store[6], &store[13])?
            .edge(&store[6], &store[14])?
            .edge(&store[7], &store[15])?
            .edge(&store[7], &store[16])?
            .edge(&store[8], &store[17])?
            .edge(&store[8], &store[18])?
            .start(&store[0])?
            .build()?;

        Ok(TreeExampleGame { game })
    }
}

impl<'a> ExampleGame<'a> for AcyclicExampleGame<'a> {
    fn new(store: &'a mut Vec<Node>) -> Result<AcyclicExampleGame<'a>> {
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
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::WIN.into(),
            ],
        ];

        let length = store.len();
        store.append(&mut nodes);
        let store = &store[length..];

        let game = builder::SessionBuilder::new(&ACYCLIC_GAME_NAME)
            .edge(&store[0], &store[1])?
            .edge(&store[0], &store[2])?
            .edge(&store[0], &store[3])?
            .edge(&store[1], &store[4])?
            .edge(&store[1], &store[5])?
            .edge(&store[2], &store[4])?
            .edge(&store[2], &store[6])?
            .edge(&store[3], &store[4])?
            .edge(&store[3], &store[7])?
            .edge(&store[3], &store[8])?
            .edge(&store[4], &store[9])?
            .edge(&store[4], &store[10])?
            .edge(&store[5], &store[11])?
            .edge(&store[5], &store[12])?
            .edge(&store[6], &store[12])?
            .edge(&store[6], &store[13])?
            .edge(&store[7], &store[10])?
            .edge(&store[8], &store[11])?
            .start(&store[0])?
            .build()?;

        Ok(AcyclicExampleGame { game })
    }
}

impl<'a> ExampleGame<'a> for CyclicExampleGame<'a> {
    fn new(store: &'a mut Vec<Node>) -> Result<CyclicExampleGame<'a>> {
        let mut nodes = vec![
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::LOSE.into(),
            ],
        ];

        let length = store.len();
        store.append(&mut nodes);
        let store = &store[length..];

        let game = builder::SessionBuilder::new(&CYCLIC_GAME_NAME)
            .edge(&store[0], &store[1])?
            .edge(&store[0], &store[3])?
            .edge(&store[1], &store[2])?
            .edge(&store[1], &store[4])?
            .edge(&store[2], &store[5])?
            .edge(&store[2], &store[7])?
            .edge(&store[3], &store[4])?
            .edge(&store[3], &store[6])?
            .edge(&store[4], &store[7])?
            .edge(&store[4], &store[9])?
            .edge(&store[5], &store[6])?
            .edge(&store[5], &store[8])?
            .edge(&store[6], &store[9])?
            .edge(&store[6], &store[11])?
            .edge(&store[7], &store[8])?
            .edge(&store[7], &store[10])?
            .edge(&store[8], &store[11])?
            .edge(&store[8], &store[13])?
            .edge(&store[9], &store[12])?
            .edge(&store[9], &store[14])?
            .edge(&store[10], &store[11])?
            .edge(&store[10], &store[13])?
            .edge(&store[11], &store[14])?
            .edge(&store[11], &store[1])?
            .edge(&store[12], &store[0])?
            .edge(&store[12], &store[13])?
            .start(&store[0])?
            .build()?;

        Ok(CyclicExampleGame { game })
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
