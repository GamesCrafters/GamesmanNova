//! Mock Game Implementations
//!
//! This module provides a way to represent extensive-form games by declaring
//! the game via a graph and assigning special conditions to nodes. This makes
//! creating example games a matter of simply declaring them and wrapping them
//! in any necessary external interface implementations.

use anyhow::Result;
use anyhow::bail;
use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::vec::BitVec;
use modular_bitfield::Specifier;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::B15;
use modular_bitfield::prelude::B32;
use petgraph::Direction;
use petgraph::Graph;
use petgraph::csr::DefaultIx;
use petgraph::dot::Config;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use rusqlite::Statement;
use rusqlite::params_from_iter;

use std::fmt::Display;
use std::sync::Arc;

use crate::database::Schema;
use crate::database::traits::DrawRecord;
use crate::database::traits::IntegerUtilityRecord;
use crate::database::traits::PlayerRecord;
use crate::database::traits::RemotenessRecord;
use crate::database::traits::RocksDBManager;
use crate::database::traits::SQLiteManager;
use crate::developer::visualize_graph;
use crate::game::IUtility;
use crate::game::Player;
use crate::game::PlayerCount;
use crate::game::Remoteness;
use crate::game::State;
use crate::game::Variant;
use crate::game::traits::Implicit;
use crate::game::traits::IntegerUtility;
use crate::game::traits::Partition;
use crate::game::traits::Sequential;
use crate::game::traits::Transpose;
use crate::game::traits::Variable;
use crate::game::util::min_ubits;

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
#[derive(Clone, Debug)]
pub enum Node {
    Terminal(Player, Vec<IUtility>),
    Medial(Player),
}

/* API STRUCTURES */

#[derive(Clone)]
pub struct Session {
    players: PlayerCount,
    rocksdb: Arc<rocksdb::DB>,
    source: NodeIndex,
    schema: Schema,
    game: Graph<Node, ()>,
    name: &'static str,
}

#[derive(Clone)]
pub struct Record {
    features: RecordFeatures,
    utility: Vec<IUtility>,
}

/* PRIVATE STRUCTURES */

#[bitfield]
#[derive(Clone, Copy, Default)]
struct RecordFeatures {
    remoteness: RemotenessStorage,
    player: PlayerStorage,
    draw: DrawStorage,
}

/* IMPLEMENTATIONS */

impl Session {
    /// Return a name or identifier corresponding to this game.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Return the number of players in this game.
    pub fn players(&self) -> PlayerCount {
        self.players
    }

    /// Return an immutable borrow of the graph underlying the game.
    pub fn graph(&self) -> &Graph<Node, ()> {
        &self.game
    }

    /// Creates an SVG visualization of the game graph in the visuals directory
    /// under the development data directory at the project root.
    pub fn visualize(&self, module: &str) -> Result<()> {
        let graph = format!("{}", self);
        visualize_graph(&graph, self.name(), module)
    }

    /* PRIVATE HELPERS */

    fn neighbors(&self, state: &State, dir: Direction) -> Vec<State> {
        let index: DefaultIx = state.load_be::<u64>() as DefaultIx;
        self.game
            .neighbors_directed(NodeIndex::from(index), dir)
            .map(|n| {
                let mut state: BitVec<u8, Msb0> = BitVec::repeat(false, 64);
                state.store_be(n.index() as u64);
                state
            })
            .collect()
    }

    fn node(&self, state: &State) -> &Node {
        let index: DefaultIx = state.load_be::<u64>() as DefaultIx;
        &self.game[NodeIndex::from(index)]
    }
}

/* IMPL TRAIT FOR TYPE */

impl Default for Node {
    fn default() -> Self {
        Self::Medial(0usize)
    }
}

impl Implicit for Session {
    fn outgoing(&self, state: &State) -> Vec<State> {
        self.neighbors(state, Direction::Outgoing)
    }

    fn source(&self) -> State {
        let mut state: BitVec<u8, Msb0> = BitVec::repeat(false, 64);
        state.store_be(self.source.index() as u64);
        state
    }

    fn sink(&self, state: &State) -> bool {
        match self.node(state) {
            Node::Terminal(_, _) => true,
            Node::Medial(_) => false,
        }
    }
}

impl Transpose for Session {
    fn incoming(&self, state: &State) -> Vec<State> {
        self.neighbors(state, Direction::Incoming)
    }
}

impl Variable for Session {
    fn variant(
        _variant: Option<Variant>,
    ) -> Result<Self> {
        anyhow::bail!("Mock games cannot be created from variant strings")
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl Partition for Session {
    fn component(&self, _state: &State) -> crate::game::Component {
        0
    }
}

impl<const N: PlayerCount> Sequential<N> for Session {
    fn turn(&self, state: &State) -> Player {
        match self.node(state) {
            Node::Terminal(player, _) => *player,
            Node::Medial(player) => *player,
        }
    }
}

impl<const N: PlayerCount> IntegerUtility<N> for Session {
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

impl<const N: PlayerCount> RocksDBManager<N> for Session {
    type Record = self::Record;

    fn rocksdb_transaction(&self) -> Result<Arc<rocksdb::DB>> {
        Ok(Arc::clone(&self.rocksdb))
    }
}

impl<const N: PlayerCount> SQLiteManager<N> for Session {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn store_lift(
        &mut self,
        state: &State,
        solution: &Self::Record,
        statement: &mut Statement,
    ) -> Result<()> {
        let mut state_bytes = [0u8; 8];
        let raw = state.as_raw_slice();
        state_bytes[..raw.len().min(8)].copy_from_slice(&raw[..raw.len().min(8)]);

        let values = [
            i64::from_be_bytes(state_bytes),
            solution.get_remoteness() as i64,
            solution.get_player() as i64,
        ]
        .into_iter()
        .chain(solution.utility.iter().copied());
        let params = params_from_iter(values);
        statement.execute(params)?;
        Ok(())
    }
}

impl Default for Record {
    fn default() -> Self {
        Self {
            features: Default::default(),
            utility: Vec::new(),
        }
    }
}

impl From<Record> for Vec<u8> {
    fn from(val: Record) -> Self {
        let mut bytes = val.features.into_bytes().to_vec();
        for util in val.utility {
            bytes.extend_from_slice(&util.to_be_bytes());
        }

        bytes
    }
}

impl RemotenessRecord for Record {
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

impl IntegerUtilityRecord for Record {
    fn set_utility(&mut self, value: Vec<IUtility>) -> Result<&mut Self> {
        self.utility = value;
        Ok(self)
    }

    fn get_utility(&self) -> Vec<IUtility> {
        self.utility.clone()
    }
}

impl PlayerRecord for Record {
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

impl DrawRecord for Record {
    fn set_draw(&mut self, value: bool) -> Result<&mut Self> {
        self.features.set_draw(value);
        Ok(self)
    }

    fn get_draw(&self) -> bool {
        self.features.draw()
    }
}

/* IMPL EXTERNAL TRAIT */

impl Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            Dot::with_attr_getters(
                &self.graph(),
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, _| String::new(),
                &|_, n| {
                    let (index, node) = n;
                    let mut attrs = String::new();
                    match node {
                        Node::Medial(turn) => {
                            attrs += &format!("label=P{turn} ");
                            attrs += "style=filled  ";
                            if index == self.source {
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

    use std::collections::HashSet;

    use anyhow::Result;

    use crate::developer::GraphBuilder;
    use crate::game::mock::SessionBuilder;
    use crate::game::traits::Implicit;
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

        let mut visited = HashSet::new();
        let mut stack = vec![g.source()];

        while let Some(state) = stack.pop() {
            if visited.insert(state.clone()) {
                stack.extend(g.outgoing(&state));
            }
        }

        assert_eq!(visited.len(), 7);

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

        let source = g.source();
        assert!(!g.sink(&source));

        let mut visited = HashSet::new();
        let mut stack = vec![source];
        let mut sinks = Vec::new();

        while let Some(state) = stack.pop() {
            if visited.insert(state.clone()) {
                if g.sink(&state) {
                    sinks.push(state);
                } else {
                    stack.extend(g.outgoing(&state));
                }
            }
        }

        assert_eq!(sinks.len(), 2);

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

        let source = g.source();
        let children = g.outgoing(&source);

        assert_eq!(children.len(), 2);

        for child in &children {
            assert!(!g.sink(child));
            let grandchildren = g.outgoing(child);

            assert_eq!(grandchildren.len(), 1);
            assert!(g.sink(&grandchildren[0]));
        }

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
