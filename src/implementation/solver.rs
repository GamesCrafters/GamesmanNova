//! # Solver Implementations
//!
//! TODO

use anyhow::Result;

use std::ops::Not;

use crate::application::database::sled;
use crate::application::solver::routines;
use crate::application::solver::scheduler;
use crate::interface::game::ClassicGame;
use crate::interface::game::ClassicPuzzle;
use crate::interface::game::Implicit;
use crate::interface::game::IntegerUtility;
use crate::interface::game::Partition;
use crate::interface::game::Sequential;
use crate::interface::game::SimpleUtility;
use crate::interface::game::Transpose;
use crate::interface::game::Variable;
use crate::model::error::SolverError;
use crate::model::frontend::IOMode;
use crate::model::game::Component;
use crate::model::game::IUtility;
use crate::model::game::PlayerCount;
use crate::model::game::SUtility;
use crate::model::game::State;
use crate::model::solver::Frontier;
use crate::model::solver::Manager;
use crate::model::solver::Task;

/* BLANKET IMPLEMENTATIONS */

// All N-player simple-utility games are also N-player integer-utility games.
impl<const N: PlayerCount, const B: usize, G> IntegerUtility<N, B> for G
where
    G: SimpleUtility<N, B>,
{
    fn utility(&self, state: &State<B>) -> [IUtility; N] {
        let sutility = self.utility(state);
        let mut iutility = [0; N];
        iutility
            .iter_mut()
            .enumerate()
            .for_each(|(i, u)| *u = IUtility::from(sutility[i]) - 1);

        iutility
    }
}

// All 2-player zero-sum games are also 2-player simple-utility games.
impl<const B: usize, G> SimpleUtility<2, B> for G
where
    G: ClassicGame<B>,
{
    fn utility(&self, state: &State<B>) -> [SUtility; 2] {
        let mut sutility = [SUtility::Tie; 2];
        let utility = self.utility(state);
        let turn = self.turn(state);
        let them = (turn + 1) % 2;
        sutility[them] = !utility;
        sutility[turn] = utility;
        sutility
    }
}

// All puzzles are also 1-player simple-utility games.
impl<const B: usize, G> SimpleUtility<1, B> for G
where
    G: ClassicPuzzle<B>,
{
    fn utility(&self, state: &State<B>) -> [SUtility; 1] {
        [self.utility(*state)]
    }
}

/* MANAGER IMPLEMENTATIONS */

trait NewTrait<G, const N: PlayerCount, const B: usize>
where
    G: Implicit<B>
        + IntegerUtility<N, B>
        + Sequential<N, B>
        + Transpose<B>
        + Clone
        + Send
        + Sync
        + 'static,
{
    fn new(game: G, mode: IOMode) -> Self;
}

impl<G, const N: PlayerCount, const B: usize> NewTrait<G, N, B>
    for Manager<G, N, B>
where
    G: Implicit<B>
        + IntegerUtility<N, B>
        + Sequential<N, B>
        + Transpose<B>
        + Clone
        + Send
        + Sync
        + 'static,
{
    fn new(game: G, mode: IOMode) -> Self {
        Self { game, mode }
    }
}

impl<G, const N: PlayerCount, const B: usize> scheduler::Manager<B>
    for Manager<G, N, B>
where
    G: Implicit<B>
        + IntegerUtility<N, B>
        + Sequential<N, B>
        + Transpose<B>
        + Partition<B>
        + Variable
        + Clone
        + Send
        + Sync
        + 'static,
{
    fn source(&self) -> Task<B> {
        let source = self.game.source();
        let component = self.game.component(&source);
        Task::Discover(component, vec![source])
    }

    fn explore(
        &self,
        comp: Component,
        front: Frontier<B>,
    ) -> Result<Vec<Task<B>>> {
        let mut tree = sled::open_tree(&self.game)?;
        routines::dfs::<G, B>(front, comp, &mut tree, &self.game)
    }

    fn process(
        &self,
        comp: Component,
        front: Frontier<B>,
    ) -> Result<Vec<Task<B>>> {
        let mut tree = sled::open_tree(&self.game)?;
        routines::viter::<G, N, B>(front, comp, &mut tree, &self.game)
    }
}

/* CONVERSIONS INTO SIMPLE UTILITY */

impl TryFrom<IUtility> for SUtility {
    type Error = SolverError;

    fn try_from(v: IUtility) -> Result<Self, Self::Error> {
        match v {
            _ if v == SUtility::Lose as i64 => Ok(SUtility::Lose),
            _ if v == SUtility::Tie as i64 => Ok(SUtility::Tie),
            _ if v == SUtility::Win as i64 => Ok(SUtility::Win),
            _ => Err(SolverError::InvalidConversion {
                input_t: "Integer Utility".into(),
                output_t: "Simple Utility".into(),
                hint:
                    "Down-casting from integer to simple utility values is not \
                    stable, and relies on the internal representation used for \
                    simple utility values."
                        .into(),
            }),
        }
    }
}

impl TryFrom<i8> for SUtility {
    type Error = SolverError;

    fn try_from(v: i8) -> Result<Self, Self::Error> {
        match v {
            _ if v == SUtility::Lose as i8 => Ok(SUtility::Lose),
            _ if v == SUtility::Tie as i8 => Ok(SUtility::Tie),
            _ if v == SUtility::Win as i8 => Ok(SUtility::Win),
            _ => Err(SolverError::InvalidConversion {
                input_t: "i8".into(),
                output_t: "Simple Utility".into(),
                hint: "Down-casting from integer to simple utility values \
                    is not stable, and relies on the internal representation \
                    used for simple utility values."
                    .into(),
            }),
        }
    }
}

/* CONVERSIONS FROM SIMPLE UTILITY */

impl From<SUtility> for IUtility {
    fn from(v: SUtility) -> Self {
        match v {
            SUtility::Lose => -1,
            SUtility::Tie => 0,
            SUtility::Win => 1,
        }
    }
}

/* SIMPLE UTILITY NEGATION */

impl Not for SUtility {
    type Output = SUtility;
    fn not(self) -> Self::Output {
        match self {
            SUtility::Lose => SUtility::Win,
            SUtility::Win => SUtility::Lose,
            SUtility::Tie => SUtility::Tie,
        }
    }
}
