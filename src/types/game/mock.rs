//! # Mock Game Models
//!
//! TODO

use modular_bitfield::bitfield;
use modular_bitfield::prelude::B15;
use modular_bitfield::prelude::B32;
use petgraph::Graph;
use petgraph::prelude::NodeIndex;

use std::collections::HashMap;

use crate::types::database::Schema;
use crate::types::game::IUtility;
use crate::types::game::Player;
use crate::types::game::PlayerCount;

/* TYPE ALIASES */

type Finalized = bool;
pub type RemotenessStorage = B32;
pub type PlayerStorage = B15;
pub type DrawStorage = bool;

/* ENUEMRATIONS */

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

/* SLED DATABASE RECORD */

#[derive(Clone, Copy)]
#[bitfield]
pub struct RecordHeader {
    pub remoteness: RemotenessStorage,
    pub player: PlayerStorage,
    pub draw: DrawStorage,
}

pub struct Record<const N: PlayerCount> {
    pub header: RecordHeader,
    pub utility: [IUtility; N],
}
