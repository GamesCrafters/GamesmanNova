//! Mock Game Implementations
//!
//! This module provides a way to represent extensive-form games by declaring
//! the game via a graph and assigning special conditions to nodes. This makes
//! creating example games a matter of simply declaring them and wrapping them
//! in any necessary external interface implementations.

use anyhow::Context;
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
use rusqlite::Transaction;
use rusqlite::params_from_iter;

use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use crate::core::database::InsertQuery;
use crate::core::database::Schema;
use crate::core::developer::DevelopmentData;
use crate::core::developer::TestSetting;
use crate::core::developer::get_directory;
use crate::core::developer::test_setting;
use crate::core::frontend::IOMode;
use crate::core::game::IUtility;
use crate::core::game::Player;
use crate::core::game::PlayerCount;
use crate::core::game::Remoteness;
use crate::core::game::State;
use crate::core::game::util::min_ubits;
use crate::traits::database::DrawRecord;
use crate::traits::database::IntegerUtilityRecord;
use crate::traits::database::PlayerRecord;
use crate::traits::database::RemotenessRecord;
use crate::traits::database::SQLiteWriter;
use crate::traits::game::Implicit;
use crate::traits::game::IntegerUtility;
use crate::traits::game::Sequential;

/* SUBMODULES */

pub mod builder;

/* TYPE ALIASES */

type Finalized = bool;
pub type RemotenessStorage = B32;
pub type PlayerStorage = B15;
pub type DrawStorage = bool;

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

/* STRUCTURES */

/// Represents an initialized session of an abstract graph game. This can be
/// constructed using `SessionBuilder`.
pub struct Session<'a> {
    pub inserted: HashMap<*const Node, NodeIndex>,
    pub players: PlayerCount,
    pub source: NodeIndex,
    pub schema: Schema,
    pub game: Graph<&'a Node, ()>,
    pub name: &'static str,
}

/// Builder pattern for creating a graph game by progressively adding nodes and
/// edges and specifying a source node. Directed unweighed edges represent
/// represent state transitions, and nodes containing either turn information
/// or utility vectors store the information necessary to solve the game being
/// represented.
///
/// # Example
///
/// ```no_run
/// // Long-form node initialization
/// let s0 = Node::Medial(0);
/// let s1 = Node::Medial(1);
/// let s2 = Node::Terminal(vec![1, -1]);
///
/// // Macro node initialization (equivalent)
/// let s0 = node!(0);
/// let s1 = node!(1);
/// let s2 = node!([1, -1]);
///
/// let session = SessionBuilder::new("example")
///     .edge(&s0, &s1)?
///     .edge(&s0, &s2)?
///     .edge(&s1, &s2)?
///     .source(&s0)?
///     .build()?;
///
/// assert_eq!(session.players, 2);
/// ```
pub struct SessionBuilder<'a> {
    pub inserted: HashMap<*const Node, NodeIndex>,
    pub players: (PlayerCount, Finalized),
    pub source: Option<NodeIndex>,
    pub game: Graph<&'a Node, ()>,
    pub name: &'static str,
}

/// Sled database record header
#[derive(Clone, Copy)]
#[bitfield]
pub struct RecordHeader {
    pub remoteness: RemotenessStorage,
    pub player: PlayerStorage,
    pub draw: DrawStorage,
}

/// Sled database record
pub struct Record<const N: PlayerCount> {
    pub header: RecordHeader,
    pub utility: [IUtility; N],
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
        match test_setting()? {
            TestSetting::Correctness => return Ok(()),
            TestSetting::Development => (),
        }

        let subdir = PathBuf::from(module);
        let mut dir = get_directory(DevelopmentData::Visuals, subdir)?;
        let name = format!("{}.svg", self.name()).replace(' ', "-");

        dir.push(name);
        let file = File::create(dir)?;
        let mut dot = Command::new("dot")
            .arg("-Tsvg")
            .stdin(Stdio::piped())
            .stdout(file)
            .spawn()
            .context("Failed to execute 'dot' command.")?;

        if let Some(mut stdin) = dot.stdin.take() {
            let graph = format!("{}", self);
            stdin.write_all(graph.as_bytes())?;
        }

        dot.wait()?;
        Ok(())
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

impl<const N: PlayerCount> SQLiteWriter<N> for Session<'_> {
    type Solution = Record<N>;
    fn prepare(
        &mut self,
        tx: &mut Transaction,
        mode: IOMode,
    ) -> Result<InsertQuery> {
        let drop_sql = self.schema.drop_table_query();
        let create_sql = self.schema.create_table_query();
        match mode {
            IOMode::Constructive | IOMode::Forgetful => (),
            IOMode::Overwrite => {
                tx.execute(&drop_sql, [])
                    .context("Failed to drop existing table")?;
            },
        }

        tx.execute(&create_sql, [])
            .context("Failed to create table")?;

        Ok(self.schema.insert_query())
    }

    fn insert(
        &mut self,
        state: &State,
        solution: &Self::Solution,
        statement: &mut Statement,
    ) -> Result<()> {
        let values = [
            i64::from_be_bytes(*state),
            solution.remoteness() as i64,
            solution.player() as i64,
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

        self.header
            .set_remoteness(value as u32);

        Ok(self)
    }

    fn remoteness(&self) -> Remoteness {
        self.header.remoteness() as u64
    }
}

impl<const N: PlayerCount> IntegerUtilityRecord<N> for Record<N> {
    fn set_utility(&mut self, value: [IUtility; N]) -> Result<&mut Self> {
        self.utility = value;
        Ok(self)
    }

    fn utility(&self) -> [IUtility; N] {
        self.utility
    }
}

impl<const N: PlayerCount> PlayerRecord for Record<N> {
    fn set_player(&mut self, value: Player) -> Result<&mut Self> {
        if min_ubits(value as u128) > PlayerStorage::BITS {
            bail!("Remoteness {value} would not fit in Sled DB record.")
        }

        self.header
            .set_player(value as u16);

        Ok(self)
    }

    fn player(&self) -> Player {
        self.header.player() as usize
    }
}

impl<const N: PlayerCount> DrawRecord for Record<N> {
    fn set_draw(&mut self, value: bool) -> Result<&mut Self> {
        self.header.set_draw(value);
        Ok(self)
    }

    fn draw(&self) -> bool {
        self.header.draw()
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

    use super::*;
    use crate::core::game::mock::SessionBuilder;
    use crate::node;
    use anyhow::Result;

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
