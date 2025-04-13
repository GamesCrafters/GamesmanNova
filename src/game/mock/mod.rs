//! Mock Test Game Module
//!
//! This module provides a way to represent extensive-form games by declaring
//! the game via a graph and assigning special conditions to nodes. This makes
//! creating example games a matter of simply declaring them and wrapping them
//! in any necessary external interface implementations.

use anyhow::Result;
use bitvec::array::BitArray;
use bitvec::field::BitField;
use bitvec::order::Msb0;
use petgraph::csr::DefaultIx;
use petgraph::Direction;
use petgraph::{graph::NodeIndex, Graph};

use std::collections::HashMap;

use crate::solver::algorithm::acyclic;
use crate::solver::{Game, IUtility, IntegerUtility, Persistent};
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::State;
use crate::game::Implicit;
use crate::game::Transpose;

/* RE-EXPORTS */

pub use builder::SessionBuilder;

/* SUBMODULES */

mod builder;

/* DEFINITIONS */

/// Represents an initialized session of an abstract graph game. This can be
/// constructed using `SessionBuilder`.
pub struct Session<'a> {
    inserted: HashMap<*const Node, NodeIndex>,
    players: PlayerCount,
    source: NodeIndex<DefaultIx>,
    game: Graph<&'a Node, ()>,
    name: &'static str,
}

/// Indicates whether a game state node is terminal (there are no outgoing moves
/// or edges) or medial (it is possible to transition out of it). Nodes in the
/// terminal stage have an associated utility vector, and medial nodes have a
/// turn encoding whose player's action is pending.
#[derive(Debug)]
pub enum Node {
    Terminal(Player, Vec<IUtility>),
    Medial(Player),
}

/* API IMPLEMENTATION */

impl<'a> Session<'a> {
    /// Return a name or identifier corresponding to this game.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Return the number of players in this game.
    pub fn players(&self) -> PlayerCount {
        self.players
    }

    /// Return the state hash being internally used for `node`.
    pub fn state(&self, node: &Node) -> Option<State> {
        if let Some(index) = self
            .inserted
            .get(&(node as *const Node))
        {
            let mut state = BitArray::<_, Msb0>::ZERO;
            state.store_be::<DefaultIx>(index.index() as DefaultIx);
            Some(state.data)
        } else {
            None
        }
    }

    /// Return an immutable borrow of the graph underlying the game.
    pub fn graph(&self) -> &Graph<&Node, ()> {
        &self.game
    }
}

/* PRIVATE IMPLEMENTATION */

impl Session<'_> {
    /// Return the states adjacent to `state`, where `dir` specifies whether
    /// they should be connected by incoming or outgoing edges.
    fn transition(&self, state: State, dir: Direction) -> Vec<State> {
        self.game
            .neighbors_directed(
                NodeIndex::from(
                    BitArray::<_, Msb0>::from(state).load_be::<DefaultIx>(),
                ),
                dir,
            )
            .map(|n| {
                let mut state: BitArray<_, Msb0> = BitArray::ZERO;
                state.store_be(n.index());
                state.data
            })
            .collect()
    }

    /// Returns a reference to the game node with `state`, or panics if there is
    /// no such node.
    fn node(&self, state: State) -> &Node {
        self.game[NodeIndex::from(
            BitArray::<_, Msb0>::from(state).load_be::<DefaultIx>(),
        )]
    }
}

/* UTILITY IMPLEMENTATIONS */

impl Implicit for Session<'_> {
    fn adjacent(&self, state: State) -> Vec<State> {
        self.transition(state, Direction::Outgoing)
    }

    fn source(&self) -> State {
        let mut state = BitArray::<_, Msb0>::ZERO;
        state.store_be::<DefaultIx>(self.source.index() as DefaultIx);
        state.data
    }

    fn sink(&self, state: State) -> bool {
        match self.node(state) {
            Node::Terminal(_, _) => true,
            Node::Medial(_) => false,
        }
    }

}

impl Transpose for Session<'_> {
    fn adjacent(&self, state: State) -> Vec<State> {
        self.transition(state, Direction::Incoming)
    }
}

/* SOLVING IMPLEMENTATIONS */

impl<const N: PlayerCount> Game<N> for Session<'_> {
    fn turn(&self, state: State) -> Player {
        match self.node(state) {
            Node::Terminal(player, _) => *player,
            Node::Medial(player) => *player,
        }
    }
} 

impl<const N: PlayerCount> IntegerUtility<N> for Session<'_> {
    fn utility(&self, state: State) -> [IUtility; N] {
        todo!()
    }
} 

impl<const N: PlayerCount> Persistent<acyclic::Solution<N>> for Session<'_> {
    fn persist(
        &self, 
        state: &State, 
        info: &acyclic::Solution<N>
    ) -> Result<()> {
        todo!()
    }

    fn retrieve(&self, state: &State) -> Result<Option<acyclic::Solution<N>>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::node;
    use anyhow::Result;

    /// Used for storing generated visualizations of the mock games being used
    /// for testing purposes in this module under their own subdirectory.
    const MODULE_NAME: &str = "mock-core-tests";

    #[test]
    fn get_unique_node_states() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(2);
        let s4 = node!(1);
        let s5 = node!(0);

        let t1 = node![0; 1, 2, 3];
        let t2 = node![1; 3, 2, 1];

        let g = SessionBuilder::new("sample1")
            .edge(&s1, &s2)?
            .edge(&s2, &s3)?
            .edge(&s3, &s4)?
            .edge(&s4, &s5)?
            .edge(&s4, &t1)?
            .edge(&s5, &t2)?
            .source(&s1)?
            .build()?;

        g.visualize(MODULE_NAME)?;
        let states = [
            g.state(&s1),
            g.state(&s2),
            g.state(&s3),
            g.state(&s4),
            g.state(&s5),
            g.state(&t1),
            g.state(&t2),
        ];

        let contains_none = states.iter().any(Option::is_none);
        assert!(!contains_none);

        let states: Vec<State> = states
            .iter()
            .map(|s| s.unwrap())
            .collect();

        let repeats = states.iter().any(|&i| {
            states[(1 + BitArray::<_, Msb0>::from(i).load_be::<usize>())..]
                .iter()
                .any(|&j| i == j)
        });

        assert!(!repeats);
        Ok(())
    }

    #[test]
    fn verify_source_and_sink_states() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(2);

        let t1 = node![2; 1, 2, 3];
        let t2 = node![1; 3, 2, 1];

        let g = SessionBuilder::new("sample2")
            .edge(&s1, &s2)?
            .edge(&s2, &s3)?
            .edge(&s2, &t1)?
            .edge(&s3, &t2)?
            .source(&s1)?
            .build()?;

        g.visualize(MODULE_NAME)?;
        let source = g.state(&s1).unwrap();
        let sink1 = g.state(&t1).unwrap();
        let sink2 = g.state(&t2).unwrap();

        assert_eq!(g.source(), source);
        assert!(g.sink(sink1));
        assert!(g.sink(sink2));
        Ok(())
    }

    #[test]
    fn verify_state_transition() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(2);

        let t1 = node![1; 1, 2, 3];
        let t2 = node![2; 3, 2, 1];

        let g = SessionBuilder::new("sample3")
            .edge(&s1, &s2)?
            .edge(&s1, &s3)?
            .edge(&s2, &t1)?
            .edge(&s3, &t2)?
            .source(&s1)?
            .build()?;

        g.visualize(MODULE_NAME)?;
        let s1_state = g.state(&s1).unwrap();
        let s2_state = g.state(&s2).unwrap();
        let s3_state = g.state(&s3).unwrap();

        let t1_state = g.state(&t1).unwrap();
        let t2_state = g.state(&t2).unwrap();

        let s1_pro = g.transition(s1_state, Direction::Outgoing);
        let s2_pro = g.transition(s2_state, Direction::Outgoing);
        let t2_ret = g.transition(t2_state, Direction::Incoming);

        assert!(s1_pro.len() == 2);
        assert!(s2_pro.len() == 1);
        assert!(t2_ret.len() == 1);

        assert!(s1_pro.contains(&s3_state));
        assert!(s1_pro.contains(&s2_state));

        assert!(s2_pro.contains(&t1_state));
        assert!(t2_ret.contains(&s3_state));

        Ok(())
    }

    #[test]
    fn get_game_name() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let t1 = node![1; -1, 2];
        let g = SessionBuilder::new("interesting name")
            .edge(&s1, &s2)?
            .edge(&s2, &t1)?
            .source(&s1)?
            .build()?;

        g.visualize(MODULE_NAME)?;
        assert_eq!(g.name(), "interesting name");
        Ok(())
    }

    #[test]
    fn get_player_count() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(5);
        let t1 = node![4; 1, -2, 3, -4, 5, -6, 7];
        let g = SessionBuilder::new("7 player game")
            .edge(&s1, &s2)?
            .edge(&s2, &t1)?
            .source(&s1)?
            .build()?;

        g.visualize(MODULE_NAME)?;
        assert_eq!(g.players(), 7);
        Ok(())
    }
}
