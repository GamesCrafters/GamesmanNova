//! # Solver Backend
//!
//! TODO

use std::sync::Arc;

use anyhow::Result;
use scheduler::Scheduler;

use crate::db::sled;
use crate::game::DEFAULT_STATE_BYTES;
use crate::game::Implicit;
use crate::game::PlayerCount;
use crate::game::State;
use crate::game::Transpose;
use crate::game::Variable;
use crate::interface::IOMode;
use crate::solver::IntegerUtility;
use crate::solver::Partition;
use crate::solver::Sequential;

use super::Component;

/* MODULES */

pub mod record;
pub mod routines;
pub mod scheduler;

/* DEIFNITIONS */

type Frontier<const B: usize = DEFAULT_STATE_BYTES> = Vec<State<B>>;

struct Manager<G, const N: PlayerCount, const B: usize = DEFAULT_STATE_BYTES>
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
    mode: IOMode,
    game: G,
}

/* API */

fn solve<G, const N: PlayerCount, const B: usize>(
    game: &G,
    mode: IOMode,
) -> Result<()>
where
    G: Implicit<B>
        + IntegerUtility<N, B>
        + Sequential<N, B>
        + Partition<B>
        + Transpose<B>
        + Variable
        + Clone
        + Send
        + Sync
        + 'static,
{
    match mode {
        IOMode::Forgetful | IOMode::Constructive => (),
        IOMode::Overwrite => sled::drop_tree(game)?,
    }

    let mut scheduler = Scheduler::<B>::new(num_cpus::get());
    let manager = Manager::<G, N, B>::new(game.clone(), mode);
    scheduler.run(Arc::new(manager));
    Ok(())
}

/* IMPLEMENTATIONS */

impl<G, const N: PlayerCount, const B: usize> Manager<G, N, B>
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
    fn source(&self) -> scheduler::Task<B> {
        let source = self.game.source();
        let component = self.game.component(&source);
        scheduler::Task::Discover(component, vec![source])
    }

    fn explore(
        &self,
        comp: Component,
        front: Frontier<B>,
    ) -> Result<Vec<scheduler::Task<B>>> {
        let mut tree = sled::open_tree(&self.game)?;
        routines::dfs::<G, B>(front, comp, &mut tree, &self.game)
    }

    fn process(
        &self,
        comp: Component,
        front: Frontier<B>,
    ) -> Result<Vec<scheduler::Task<B>>> {
        let mut tree = sled::open_tree(&self.game)?;
        routines::viter::<G, N, B>(front, comp, &mut tree, &self.game)
    }
}
