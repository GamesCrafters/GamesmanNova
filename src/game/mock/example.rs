//! # Example Mock Test Game Module
//!
//! This module provides concrete examples of small games that adhere to useful
//! interface definitions that can be used for testing purposes. The games here
//! are built over the `mock` game graph implementation.
//!
//! #### Authorship
//! - Max Fierro, 4/8/2024

/* CONSTANTS */

/// Specifies a directory name under which to store visualizations of the games
/// declared in this module.
const MODULE_STORAGE: &str = "mock-game-examples";

/* CATEGORIES */

/// Contains examples of mock games where utility can be expressed as `WIN`,
/// `LOSE`, `TIE`, or `DRAW` for players.
pub mod simple_utility {

    /* CATEGORIES */

    /// Contains examples of mock games where payoffs for terminal states can
    /// sum to anything across all players.
    pub mod general_sum {

        use anyhow::Result;

        use crate::game::{mock::*, SimpleSum};
        use crate::model::SimpleUtility;
        use crate::node;

        /* CONSTANTS */

        const TREE_GAME_NAME: &str =
            "simple-utility-general-sum-tree-structure";
        const ACYCLIC_GAME_NAME: &str =
            "simple-utility-general-sum-acyclic-structure";
        const CYCLIC_GAME_NAME: &str =
            "simple-utility-general-sum-cyclic-structure";

        /* DEFINITIONS */

        /// TODO
        pub struct TreeExampleGame<'a> {
            game: Session<'a>,
        }

        /// TODO
        pub struct AcyclicExampleGame<'a> {
            game: Session<'a>,
        }

        /// TODO
        pub struct CyclicExampleGame<'a> {
            game: Session<'a>,
        }

        /* INSTANTIATION */

        impl<'a> TreeExampleGame<'a> {
            /// Creates a new [`TreeExampleGame`] by instantiating all of its
            /// nodes and appending them to `store` non-destructively (even if
            /// the function fails). The example game returned will reference
            /// the nodes added to `store` internally, so removing them from
            /// `store` and attempting to use the returned game after is
            /// undefined behavior.
            pub fn new(
                store: &'a mut Vec<Node>,
            ) -> Result<TreeExampleGame<'a>> {
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
                        SimpleUtility::WIN.into(),
                    ],
                    node![
                        SimpleUtility::TIE.into(),
                        SimpleUtility::TIE.into(),
                    ],
                    node![
                        SimpleUtility::WIN.into(),
                        SimpleUtility::WIN.into(),
                    ],
                    node![
                        SimpleUtility::LOSE.into(),
                        SimpleUtility::LOSE.into(),
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
                        SimpleUtility::LOSE.into(),
                        SimpleUtility::TIE.into(),
                    ],
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

            /// Creates a PNG image of the game being represented.
            pub fn visualize(&self) -> Result<()> {
                self.game
                    .visualize(super::super::MODULE_STORAGE)
            }
        }

        impl<'a> AcyclicExampleGame<'a> {
            /// TODO
            pub fn new(
                store: &'a mut Vec<Node>,
            ) -> Result<AcyclicExampleGame<'a>> {
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
                    node!(1),
                    node!(0),
                    node!(1),
                    node!(0),
                    node!(1),
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
                    node![
                        SimpleUtility::WIN.into(),
                        SimpleUtility::TIE.into(),
                    ],
                    node![
                        SimpleUtility::WIN.into(),
                        SimpleUtility::TIE.into(),
                    ],
                    node![
                        SimpleUtility::TIE.into(),
                        SimpleUtility::TIE.into(),
                    ],
                    node![
                        SimpleUtility::WIN.into(),
                        SimpleUtility::WIN.into(),
                    ],
                    node![
                        SimpleUtility::LOSE.into(),
                        SimpleUtility::LOSE.into(),
                    ],
                ];

                let length = store.len();
                store.append(&mut nodes);
                let store: &_ = &store[length..];

                let game = builder::SessionBuilder::new(&ACYCLIC_GAME_NAME)
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
                    .edge(&store[9], &store[10])?
                    .edge(&store[9], &store[12])?
                    .edge(&store[10], &store[13])?
                    .edge(&store[10], &store[15])?
                    .edge(&store[11], &store[12])?
                    .edge(&store[11], &store[14])?
                    .edge(&store[12], &store[15])?
                    .edge(&store[12], &store[17])?
                    .edge(&store[13], &store[14])?
                    .edge(&store[13], &store[16])?
                    .edge(&store[14], &store[17])?
                    .edge(&store[14], &store[19])?
                    .edge(&store[15], &store[16])?
                    .edge(&store[15], &store[18])?
                    .edge(&store[16], &store[19])?
                    .edge(&store[16], &store[20])?
                    .edge(&store[17], &store[21])?
                    .edge(&store[17], &store[22])?
                    .start(&store[0])?
                    .build()?;

                Ok(AcyclicExampleGame { game })
            }

            /// Creates a PNG image of the game being represented.
            pub fn visualize(&self) -> Result<()> {
                self.game
                    .visualize(super::super::MODULE_STORAGE)
            }
        }

        impl<'a> CyclicExampleGame<'a> {
            /// TODO
            pub fn new(
                store: &'a mut Vec<Node>,
            ) -> Result<CyclicExampleGame<'a>> {
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
                    node!(1),
                    node!(0),
                    node![
                        SimpleUtility::LOSE.into(),
                        SimpleUtility::LOSE.into(),
                    ],
                    node![
                        SimpleUtility::WIN.into(),
                        SimpleUtility::LOSE.into(),
                    ],
                    node![
                        SimpleUtility::TIE.into(),
                        SimpleUtility::WIN.into(),
                    ],
                    node![
                        SimpleUtility::LOSE.into(),
                        SimpleUtility::WIN.into(),
                    ],
                    node![
                        SimpleUtility::LOSE.into(),
                        SimpleUtility::LOSE.into(),
                    ],
                    node![
                        SimpleUtility::WIN.into(),
                        SimpleUtility::LOSE.into(),
                    ],
                    node![
                        SimpleUtility::TIE.into(),
                        SimpleUtility::WIN.into(),
                    ],
                ];

                let length = store.len();
                store.append(&mut nodes);
                let store = &store[length..];

                let game = builder::SessionBuilder::new(&CYCLIC_GAME_NAME)
                    .edge(&store[0], &store[1])?
                    .edge(&store[0], &store[3])?
                    .edge(&store[0], &store[5])?
                    .edge(&store[1], &store[8])?
                    .edge(&store[1], &store[10])?
                    .edge(&store[2], &store[1])?
                    .edge(&store[2], &store[3])?
                    .edge(&store[2], &store[5])?
                    .edge(&store[3], &store[2])?
                    .edge(&store[3], &store[4])?
                    .edge(&store[4], &store[5])?
                    .edge(&store[4], &store[7])?
                    .edge(&store[5], &store[6])?
                    .edge(&store[5], &store[8])?
                    .edge(&store[6], &store[7])?
                    .edge(&store[6], &store[9])?
                    .edge(&store[6], &store[16])?
                    .edge(&store[7], &store[11])?
                    .edge(&store[7], &store[12])?
                    .edge(&store[7], &store[4])?
                    .edge(&store[8], &store[9])?
                    .edge(&store[8], &store[14])?
                    .edge(&store[9], &store[13])?
                    .edge(&store[10], &store[15])?
                    .edge(&store[10], &store[13])?
                    .start(&store[0])?
                    .build()?;

                Ok(CyclicExampleGame { game })
            }

            /// Creates a PNG image of the game being represented.
            pub fn visualize(&self) -> Result<()> {
                self.game
                    .visualize(super::super::MODULE_STORAGE)
            }
        }

        /* TRAVERSAL IMPLEMENTATIONS */

        impl MockGame for TreeExampleGame<'_> {
            fn game(&self) -> &Session<'_> {
                &self.game
            }
        }

        impl MockGame for AcyclicExampleGame<'_> {
            fn game(&self) -> &Session<'_> {
                &self.game
            }
        }

        impl MockGame for CyclicExampleGame<'_> {
            fn game(&self) -> &Session<'_> {
                &self.game
            }
        }

        /* TREE GAME UTILITY IMPLEMENTATIONS */

        impl SimpleSum<2> for TreeExampleGame<'_> {
            fn utility(&self, state: State) -> [SimpleUtility; 2] {
                match self.game.node(state) {
                    Node::Terminal(vector) => [
                        vector[0].try_into().unwrap(),
                        vector[1].try_into().unwrap(),
                    ],
                    Node::Medial(_) => {
                        panic!("Attempted to fetch utility of medial state.")
                    },
                }
            }
        }
    }

    /// Contains examples of mock games where payoffs for terminal states always
    /// sum to zero across all players.
    pub mod zero_sum {

        use anyhow::Result;

        use crate::game::{mock::*, SimpleSum};
        use crate::model::SimpleUtility;
        use crate::node;

        /* CONSTANTS */

        const TREE_GAME_NAME: &str = "simple-utility-zero-sum-tree-structure";
        const ACYCLIC_GAME_NAME: &str =
            "simple-utility-zero-sum-acyclic-structure";
        const CYCLIC_GAME_NAME: &str =
            "simple-utility-zero-sum-cyclic-structure";

        /* DEFINITIONS */

        /// TODO
        pub struct TreeExampleGame<'a> {
            game: Session<'a>,
        }

        /// TODO
        pub struct AcyclicExampleGame<'a> {
            game: Session<'a>,
        }

        /// TODO
        pub struct CyclicExampleGame<'a> {
            game: Session<'a>,
        }

        /* INSTANTIATION */

        impl<'a> TreeExampleGame<'a> {
            pub fn new(
                store: &'a mut Vec<Node>,
            ) -> Result<TreeExampleGame<'a>> {
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

            /// Creates a PNG image of the game being represented.
            pub fn visualize(&self) -> Result<()> {
                self.game
                    .visualize(super::super::MODULE_STORAGE)
            }
        }

        impl<'a> AcyclicExampleGame<'a> {
            pub fn new(
                store: &'a mut Vec<Node>,
            ) -> Result<AcyclicExampleGame<'a>> {
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

            /// Creates a PNG image of the game being represented.
            pub fn visualize(&self) -> Result<()> {
                self.game
                    .visualize(super::super::MODULE_STORAGE)
            }
        }

        impl<'a> CyclicExampleGame<'a> {
            /// TODO
            pub fn new(
                store: &'a mut Vec<Node>,
            ) -> Result<CyclicExampleGame<'a>> {
                todo!()
            }

            /// Creates a PNG image of the game being represented.
            pub fn visualize(&self) -> Result<()> {
                self.game
                    .visualize(super::super::MODULE_STORAGE)
            }
        }

        /* TRAVERSAL IMPLEMENTATIONS */

        impl MockGame for TreeExampleGame<'_> {
            fn game(&self) -> &Session<'_> {
                &self.game
            }
        }

        impl MockGame for AcyclicExampleGame<'_> {
            fn game(&self) -> &Session<'_> {
                &self.game
            }
        }

        impl MockGame for CyclicExampleGame<'_> {
            fn game(&self) -> &Session<'_> {
                &self.game
            }
        }
    }
}

/// Contains examples of mock games where utility can be expressed as an general
/// for players.
pub mod general_utility {

    /* CATEGORIES */

    /// Contains examples of mock games where payoffs for terminal states can
    /// sum to anything across all players.
    pub mod general_sum {

        use anyhow::Result;

        use crate::game::{mock::*, SimpleSum};
        use crate::model::SimpleUtility;
        use crate::node;

        /* CONSTANTS */

        const TREE_GAME_NAME: &str =
            "general-utility-general-sum-tree-structure";
        const ACYCLIC_GAME_NAME: &str =
            "general-utility-general-sum-acyclic-structure";
        const CYCLIC_GAME_NAME: &str =
            "general-utility-general-sum-cyclic-structure";

        /* DEFINITIONS */

        /// TODO
        pub struct TreeExampleGame<'a> {
            game: Session<'a>,
        }

        /// TODO
        pub struct AcyclicExampleGame<'a> {
            game: Session<'a>,
        }

        /// TODO
        pub struct CyclicExampleGame<'a> {
            game: Session<'a>,
        }

        /* INSTANTIATION */

        impl<'a> TreeExampleGame<'a> {
            /// TODO
            pub fn new(
                store: &'a mut Vec<Node>,
            ) -> Result<TreeExampleGame<'a>> {
                todo!()
            }

            /// Creates a PNG image of the game being represented.
            pub fn visualize(&self) -> Result<()> {
                self.game
                    .visualize(super::super::MODULE_STORAGE)
            }
        }

        impl<'a> AcyclicExampleGame<'a> {
            /// TODO
            pub fn new(
                store: &'a mut Vec<Node>,
            ) -> Result<AcyclicExampleGame<'a>> {
                todo!()
            }

            /// Creates a PNG image of the game being represented.
            pub fn visualize(&self) -> Result<()> {
                self.game
                    .visualize(super::super::MODULE_STORAGE)
            }
        }

        impl<'a> CyclicExampleGame<'a> {
            /// TODO
            pub fn new(
                store: &'a mut Vec<Node>,
            ) -> Result<CyclicExampleGame<'a>> {
                todo!()
            }

            /// Creates a PNG image of the game being represented.
            pub fn visualize(&self) -> Result<()> {
                self.game
                    .visualize(super::super::MODULE_STORAGE)
            }
        }

        /* TRAVERSAL IMPLEMENTATIONS */

        impl MockGame for TreeExampleGame<'_> {
            fn game(&self) -> &Session<'_> {
                &self.game
            }
        }

        impl MockGame for AcyclicExampleGame<'_> {
            fn game(&self) -> &Session<'_> {
                &self.game
            }
        }

        impl MockGame for CyclicExampleGame<'_> {
            fn game(&self) -> &Session<'_> {
                &self.game
            }
        }
    }

    /// Contains examples of mock games where payoffs for terminal states always
    /// sum to zero across all players.
    pub mod zero_sum {

        use anyhow::Result;

        use crate::game::{mock::*, SimpleSum};
        use crate::model::SimpleUtility;
        use crate::node;

        /* CONSTANTS */

        const TREE_GAME_NAME: &str = "general-utility-zero-sum-tree-structure";
        const ACYCLIC_GAME_NAME: &str =
            "general-utility-zero-sum-acyclic-structure";
        const CYCLIC_GAME_NAME: &str =
            "general-utility-zero-sum-cyclic-structure";

        /* DEFINITIONS */

        /// TODO
        pub struct TreeExampleGame<'a> {
            game: Session<'a>,
        }

        /// TODO
        pub struct AcyclicExampleGame<'a> {
            game: Session<'a>,
        }

        /// TODO
        pub struct CyclicExampleGame<'a> {
            game: Session<'a>,
        }

        /* INSTANTIATION */

        impl<'a> TreeExampleGame<'a> {
            /// TODO
            pub fn new(
                store: &'a mut Vec<Node>,
            ) -> Result<TreeExampleGame<'a>> {
                todo!()
            }

            /// Creates a PNG image of the game being represented.
            pub fn visualize(&self) -> Result<()> {
                self.game
                    .visualize(super::super::MODULE_STORAGE)
            }
        }

        impl<'a> AcyclicExampleGame<'a> {
            /// TODO
            pub fn new(
                store: &'a mut Vec<Node>,
            ) -> Result<AcyclicExampleGame<'a>> {
                todo!()
            }

            /// Creates a PNG image of the game being represented.
            pub fn visualize(&self) -> Result<()> {
                self.game
                    .visualize(super::super::MODULE_STORAGE)
            }
        }

        impl<'a> CyclicExampleGame<'a> {
            /// TODO
            pub fn new(
                store: &'a mut Vec<Node>,
            ) -> Result<CyclicExampleGame<'a>> {
                todo!()
            }

            /// Creates a PNG image of the game being represented.
            pub fn visualize(&self) -> Result<()> {
                self.game
                    .visualize(super::super::MODULE_STORAGE)
            }
        }

        /* TRAVERSAL IMPLEMENTATIONS */

        impl MockGame for TreeExampleGame<'_> {
            fn game(&self) -> &Session<'_> {
                &self.game
            }
        }

        impl MockGame for AcyclicExampleGame<'_> {
            fn game(&self) -> &Session<'_> {
                &self.game
            }
        }

        impl MockGame for CyclicExampleGame<'_> {
            fn game(&self) -> &Session<'_> {
                &self.game
            }
        }
    }
}

mod tests {

    use anyhow::Result;

    use super::*;

    #[test]
    fn initialize_simple_utility_general_sum() -> Result<()> {
        let mut s = vec![];
        let _ = simple_utility::general_sum::TreeExampleGame::new(&mut s)?;
        let _ = simple_utility::general_sum::AcyclicExampleGame::new(&mut s)?;
        let _ = simple_utility::general_sum::CyclicExampleGame::new(&mut s)?;
        Ok(())
    }

    #[test]
    fn initialize_simple_utility_zero_sum() -> Result<()> {
        let mut s = vec![];
        let _ = simple_utility::zero_sum::TreeExampleGame::new(&mut s)?;
        let _ = simple_utility::zero_sum::AcyclicExampleGame::new(&mut s)?;
        let _ = simple_utility::zero_sum::CyclicExampleGame::new(&mut s)?;
        Ok(())
    }

    #[test]
    fn initialize_general_utility_general_sum() -> Result<()> {
        let mut s = vec![];
        let _ = general_utility::general_sum::TreeExampleGame::new(&mut s)?;
        let _ = general_utility::general_sum::AcyclicExampleGame::new(&mut s)?;
        let _ = general_utility::general_sum::CyclicExampleGame::new(&mut s)?;
        Ok(())
    }

    #[test]
    fn initialize_general_utility_zero_sum() -> Result<()> {
        let mut s = vec![];
        let _ = general_utility::zero_sum::TreeExampleGame::new(&mut s)?;
        let _ = general_utility::zero_sum::AcyclicExampleGame::new(&mut s)?;
        let _ = general_utility::zero_sum::CyclicExampleGame::new(&mut s)?;
        Ok(())
    }

    #[test]
    fn visualize_all_example_games() -> Result<()> {
        let mut s = vec![];
        let _ = simple_utility::general_sum::TreeExampleGame::new(&mut s)?
            .visualize();
        let mut s = vec![];
        let _ = simple_utility::general_sum::AcyclicExampleGame::new(&mut s)?
            .visualize();
        let mut s = vec![];
        let _ = simple_utility::general_sum::CyclicExampleGame::new(&mut s)?
            .visualize();

        let mut s = vec![];
        let _ =
            simple_utility::zero_sum::TreeExampleGame::new(&mut s)?.visualize();
        let mut s = vec![];
        let _ = simple_utility::zero_sum::AcyclicExampleGame::new(&mut s)?
            .visualize();
        let mut s = vec![];
        let _ = simple_utility::zero_sum::CyclicExampleGame::new(&mut s)?
            .visualize();

        let mut s = vec![];
        let _ = general_utility::general_sum::TreeExampleGame::new(&mut s)?
            .visualize();
        let mut s = vec![];
        let _ = general_utility::general_sum::AcyclicExampleGame::new(&mut s)?
            .visualize();
        let mut s = vec![];
        let _ = general_utility::general_sum::CyclicExampleGame::new(&mut s)?
            .visualize();

        let mut s = vec![];
        let _ = general_utility::zero_sum::TreeExampleGame::new(&mut s)?
            .visualize();
        let mut s = vec![];
        let _ = general_utility::zero_sum::AcyclicExampleGame::new(&mut s)?
            .visualize();
        let mut s = vec![];
        let _ = general_utility::zero_sum::CyclicExampleGame::new(&mut s)?
            .visualize();

        Ok(())
    }
}
