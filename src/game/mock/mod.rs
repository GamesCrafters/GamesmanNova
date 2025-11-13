//! Mock Game Implementations
//!
//! This module provides a way to represent extensive-form games by declaring
//! the game via a graph and assigning special conditions to nodes. This makes
//! creating example games a matter of simply declaring them and wrapping them
//! in any necessary external interface implementations.

use anyhow::Result;
use anyhow::bail;
use bitvec::array::BitArray;
use bitvec::field::BitField;
use bitvec::order::Msb0;
use modular_bitfield::Specifier;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::B15;
use modular_bitfield::prelude::B32;
use petgraph::Direction;
use petgraph::Graph;
use petgraph::csr::DefaultIx;
use petgraph::csr::IndexType;
use petgraph::dot::Config;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use rusqlite::Statement;
use rusqlite::params_from_iter;

use std::collections::HashMap;
use std::fmt::Display;

use crate::database::Schema;
use crate::database::traits::DrawRecord;
use crate::database::traits::IntegerUtilityRecord;
use crate::database::traits::PlayerRecord;
use crate::database::traits::RemotenessRecord;
use crate::database::traits::SQLiteManager;
use crate::database::traits::SQLiteWriter;
use crate::developer::visualize_graph;
use crate::game::IUtility;
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::Remoteness;
use crate::game::State;
use crate::game::traits::Implicit;
use crate::game::traits::IntegerUtility;
use crate::game::traits::Sequential;
use crate::game::util::min_ubits;

/* RE-EXPORTS */

pub use builder::SessionBuilder;

/* SUBMODULES */

mod builder;

/* TYPE ALIASES */

type RemotenessStorage = B32;
type PlayerStorage = B15;
type DrawStorage = bool;

/* ENUMERATIONS */

/// Indicates whether a game state node is terminal (there are no outgoing moves
/// or edges) or medial (it is possible to transition out of it). Nodes in the
/// terminal stage have an associated utility vector, and medial nodes have a
/// turn encoding whose player's action is pending.
#[derive(Debug)]
pub enum Node {
    Terminal(Player, Vec<IUtility>),
    Medial(Player),
}

/* API STRUCTURES */

pub struct Session<'a> {
    inserted: HashMap<*const Node, NodeIndex>,
    players: PlayerCount,
    source: NodeIndex,
    schema: Schema,
    game: Graph<&'a Node, ()>,
    name: &'static str,
}

pub struct Record<const N: PlayerCount> {
    features: RecordFeatures,
    utility: [IUtility; N],
}

/* PRIVATE STRUCTURES */

#[derive(Clone, Copy)]
#[bitfield]
struct RecordFeatures {
    remoteness: RemotenessStorage,
    player: PlayerStorage,
    draw: DrawStorage,
}

/* IMPLEMENTATIONS */

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
        self.inserted
            .get(&(node as *const Node))
            .map(|idx| {
                let mut state = BitArray::<_, Msb0>::ZERO;
                state.store_be::<DefaultIx>(idx.index() as DefaultIx);
                state.data
            })
    }

    /// Return an immutable borrow of the graph underlying the game.
    pub fn graph(&self) -> &Graph<&Node, ()> {
        &self.game
    }

    /// Creates an SVG visualization of the game graph in the visuals directory
    /// under the development data directory at the project root.
    pub fn visualize(&self, module: &str) -> Result<()> {
        let graph = format!("{}", self);
        visualize_graph(&graph, self.name(), module)
    }

    /* PRIVATE HELPERS */

    fn adjacent(&self, state: &State, dir: Direction) -> Vec<State> {
        self.game
            .neighbors_directed(
                NodeIndex::from(
                    BitArray::<_, Msb0>::from(*state).load_be::<DefaultIx>(),
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

    fn node(&self, state: &State) -> &Node {
        self.game[NodeIndex::from(
            BitArray::<_, Msb0>::from(*state).load_be::<DefaultIx>(),
        )]
    }
}

/* IMPL TRAIT FOR TYPE */

impl Default for Node {
    fn default() -> Self {
        Self::Medial(0usize)
    }
}

impl Implicit for Session<'_> {
    fn adjacent(&self, state: &State) -> Vec<State> {
        self.adjacent(state, Direction::Outgoing)
    }

    fn source(&self) -> State {
        let mut state = BitArray::<_, Msb0>::ZERO;
        state.store_be::<DefaultIx>(self.source.index() as DefaultIx);
        state.data
    }

    fn sink(&self, state: &State) -> bool {
        match self.node(state) {
            Node::Terminal(_, _) => true,
            Node::Medial(_) => false,
        }
    }
}

impl<const N: PlayerCount> Sequential<N> for Session<'_> {
    fn turn(&self, state: &State) -> Player {
        match self.node(state) {
            Node::Terminal(player, _) => *player,
            Node::Medial(player) => *player,
        }
    }
}

impl<const N: PlayerCount> IntegerUtility<N> for Session<'_> {
    fn utility(&self, state: &State) -> [IUtility; N] {
        match self.node(state) {
            Node::Terminal(_, payoffs) => {
                let mut res = [0; N];
                res[..N].copy_from_slice(&payoffs[..N]);
                res
            },
            Node::Medial(_) => {
                panic!("Attempt to fetch utility of medial state.")
            },
        }
    }
}

impl<const N: PlayerCount> SQLiteManager<N> for Session<'_> {
    type SolutionRecord = Record<N>;
    fn schema(&self) -> &Schema {
        &self.schema
    }
}

impl<const N: PlayerCount> SQLiteWriter<Record<N>, N> for Session<'_> {
    fn insert(
        &mut self,
        state: &State,
        solution: &Self::SolutionRecord,
        statement: &mut Statement,
    ) -> Result<()> {
        let values = [
            i64::from_be_bytes(*state),
            solution.get_remoteness() as i64,
            solution.get_player() as i64,
        ]
        .into_iter()
        .chain(solution.utility);
        let params = params_from_iter(values);
        statement.execute(params)?;
        Ok(())
    }
}

impl<const N: PlayerCount> RemotenessRecord for Record<N> {
    fn set_remoteness(&mut self, value: Remoteness) -> Result<&mut Self> {
        if min_ubits(value as u128) > RemotenessStorage::BITS {
            bail!("Remoteness {value} would not fit in Sled DB record.")
        }

        self.features
            .set_remoteness(value as u32);

        Ok(self)
    }

    fn get_remoteness(&self) -> Remoteness {
        self.features.remoteness() as u64
    }
}

impl<const N: PlayerCount> IntegerUtilityRecord<N> for Record<N> {
    fn set_utility(&mut self, value: [IUtility; N]) -> Result<&mut Self> {
        self.utility = value;
        Ok(self)
    }

    fn get_utility(&self) -> [IUtility; N] {
        self.utility
    }
}

impl<const N: PlayerCount> PlayerRecord for Record<N> {
    fn set_player(&mut self, value: Player) -> Result<&mut Self> {
        if min_ubits(value as u128) > PlayerStorage::BITS {
            bail!("Remoteness {value} would not fit in Sled DB record.")
        }

        self.features
            .set_player(value as u16);

        Ok(self)
    }

    fn get_player(&self) -> Player {
        self.features.player() as usize
    }
}

impl<const N: PlayerCount> DrawRecord for Record<N> {
    fn set_draw(&mut self, value: bool) -> Result<&mut Self> {
        self.features.set_draw(value);
        Ok(self)
    }

    fn get_draw(&self) -> bool {
        self.features.draw()
    }
}

/* IMPL EXTERNAL TRAIT */

impl Display for Session<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            Dot::with_attr_getters(
                &self.graph(),
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, _| String::new(),
                &|_, n| {
                    let (_, node) = n;
                    let mut attrs = String::new();
                    match node {
                        Node::Medial(turn) => {
                            attrs += &format!("label=P{turn} ");
                            attrs += "style=filled  ";
                            if self.source() == self.state(node).unwrap() {
                                attrs += "shape=doublecircle ";
                                attrs += "fillcolor=navajowhite3 ";
                            } else {
                                attrs += "shape=circle ";
                                attrs += "fillcolor=lightsteelblue ";
                            }
                        },
                        Node::Terminal(turn, util) => {
                            attrs += &format!("label=\"P{turn}, {:?}\" ", util);
                            attrs += "shape=plain ";
                        },
                    }
                    attrs
                }
            )
        )
    }
}

/* TESTS */

#[cfg(test)]
mod tests {

    use anyhow::Result;

    use crate::developer::GraphBuilder;
    use crate::game::mock::SessionBuilder;
    use crate::node;

    use super::*;

    const MODULE_NAME: &str = "mock-game-tests";

    #[test]
    fn get_unique_node_states() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(2);
        let s4 = node!(1);
        let s5 = node!(0);

        let t1 = node![0; 1, 2, 3];
        let t2 = node![1; 3, 2, 1];

        let graph = GraphBuilder::default()
            .edge(&s1, &s2)
            .edge(&s2, &s3)
            .edge(&s3, &s4)
            .edge(&s4, &s5)
            .edge(&s4, &t1)
            .edge(&s5, &t2);

        let g = SessionBuilder::default()
            .name("sample1")
            .graph(graph)
            .source(&s1)
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
                .contains(&i)
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

        let graph = GraphBuilder::default()
            .edge(&s1, &s2)
            .edge(&s2, &s3)
            .edge(&s2, &t1)
            .edge(&s3, &t2);

        let g = SessionBuilder::default()
            .name("sample2")
            .graph(graph)
            .source(&s1)
            .build()?;

        g.visualize(MODULE_NAME)?;
        let source = g.state(&s1).unwrap();
        let sink1 = g.state(&t1).unwrap();
        let sink2 = g.state(&t2).unwrap();

        assert_eq!(g.source(), source);
        assert!(g.sink(&sink1));
        assert!(g.sink(&sink2));
        Ok(())
    }

    #[test]
    fn verify_state_transition() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(2);

        let t1 = node![1; 1, 2, 3];
        let t2 = node![2; 3, 2, 1];

        let graph = GraphBuilder::default()
            .edge(&s1, &s2)
            .edge(&s1, &s3)
            .edge(&s2, &t1)
            .edge(&s3, &t2);

        let g = SessionBuilder::default()
            .name("sample3")
            .graph(graph)
            .source(&s1)
            .build()?;

        g.visualize(MODULE_NAME)?;
        let s1_state = g.state(&s1).unwrap();
        let s2_state = g.state(&s2).unwrap();
        let s3_state = g.state(&s3).unwrap();

        let t1_state = g.state(&t1).unwrap();
        let t2_state = g.state(&t2).unwrap();

        let s1_pro = g.adjacent(&s1_state, Direction::Outgoing);
        let s2_pro = g.adjacent(&s2_state, Direction::Outgoing);
        let t2_ret = g.adjacent(&t2_state, Direction::Incoming);

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

        let graph = GraphBuilder::default()
            .edge(&s1, &s2)
            .edge(&s2, &t1);

        let g = SessionBuilder::default()
            .name("interesting name")
            .graph(graph)
            .source(&s1)
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

        let graph = GraphBuilder::default()
            .edge(&s1, &s2)
            .edge(&s2, &t1);

        let g = SessionBuilder::default()
            .name("7 player game")
            .graph(graph)
            .source(&s1)
            .build()?;

        g.visualize(MODULE_NAME)?;
        assert_eq!(g.players(), 7);
        Ok(())
    }
}
