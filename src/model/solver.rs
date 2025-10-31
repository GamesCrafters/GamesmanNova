//! # Solver Models
//!
//! TODO

use crate::interface::game::Implicit;
use crate::interface::game::IntegerUtility;
use crate::interface::game::Sequential;
use crate::interface::game::Transpose;
use crate::model::frontend::IOMode;
use crate::model::game::Component;
use crate::model::game::DEFAULT_STATE_BYTES;
use crate::model::game::IUtility;
use crate::model::game::Player;
use crate::model::game::PlayerCount;
use crate::model::game::State;

/* TYPE ALIASES */

/// TODO
pub type Frontier<const B: usize = DEFAULT_STATE_BYTES> = Vec<State<B>>;

/// Indicates the number of outoing edges that exist from a given game state.
pub type Degree = u64;

/// Indicates the number of choices that players have to make to reach a
/// terminal state in a game under perfect play. For drawing positions,
/// indicates the number of choices players can make to bring the game to a
/// state which can transition to a non-drawing state.
pub type Remoteness = u64;

/* ENUMERATIONS */

#[derive(Debug, Clone)]
pub enum Task<const B: usize = DEFAULT_STATE_BYTES> {
    Discover(Component, Frontier<B>),
    Process(Component, Frontier<B>),
}

/* STRUCTURES */

/// Values that solving algorithms calculate for each state within a game.
#[derive(Debug)]
pub struct Solution<const N: PlayerCount> {
    pub remoteness: Remoteness,
    pub utility: [IUtility; N],
    pub player: Player,
}

/// TODO
pub struct Manager<
    G,
    const N: PlayerCount,
    const B: usize = DEFAULT_STATE_BYTES,
> where
    G: Implicit<B>
        + IntegerUtility<N, B>
        + Sequential<N, B>
        + Transpose<B>
        + Clone
        + Send
        + Sync
        + 'static,
{
    pub mode: IOMode,
    pub game: G,
}
