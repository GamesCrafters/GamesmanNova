//! Scheduler module
//!
//! TODO

use anyhow::Result;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use crossbeam_channel::unbounded;
use petgraph::Direction;
use petgraph::graphmap::DiGraphMap;
use threadpool::ThreadPool;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use crate::game::DEFAULT_STATE_BYTES;
use crate::game::State;
use crate::solver::Component;
use crate::solver::backend::Frontier;

/* DEFINITIONS */

#[derive(Debug, Clone)]
pub enum Task<const B: usize = DEFAULT_STATE_BYTES> {
    Discover(Component, Frontier<B>),
    Process(Component, Frontier<B>),
}

pub trait Manager<const B: usize = DEFAULT_STATE_BYTES> {
    fn source(&self) -> Task<B>;

    fn explore(
        &self,
        comp: Component,
        front: Frontier<B>,
    ) -> Result<Vec<Task<B>>>;

    fn process(
        &self,
        comp: Component,
        front: Frontier<B>,
    ) -> Result<Vec<Task<B>>>;
}

pub struct Scheduler<const B: usize = DEFAULT_STATE_BYTES> {
    pool: ThreadPool,
    graph: DiGraphMap<Component, ()>,
    pending: HashSet<Component>,
    completed: HashSet<Component>,
    scheduled: HashSet<Component>,
    accumulated: HashMap<Component, Frontier<B>>,
    task_tx: Sender<Task<B>>,
    task_rx: Receiver<Task<B>>,
}

/* IMPLEMENTATIONS */

impl<const B: usize> Task<B> {
    pub fn component(&self) -> Component {
        match self {
            Task::Discover(comp, _) | Task::Process(comp, _) => *comp,
        }
    }

    pub fn frontier(&self) -> &Frontier<B> {
        match self {
            Task::Discover(_, front) | Task::Process(_, front) => &front,
        }
    }

    pub fn push(&mut self, state: State<B>) {
        match self {
            Task::Discover(_, front) | Task::Process(_, front) => {
                front.push(state)
            },
        };
    }
}

impl<const B: usize> Scheduler<B> {
    pub fn new(threads: usize) -> Self {
        let (task_tx, task_rx) = unbounded();
        Self {
            pool: ThreadPool::new(threads),
            graph: DiGraphMap::new(),
            pending: HashSet::new(),
            completed: HashSet::new(),
            scheduled: HashSet::new(),
            accumulated: HashMap::new(),
            task_tx,
            task_rx,
        }
    }

    pub fn run<M>(&mut self, manager: Arc<M>)
    where
        M: Manager<B> + Send + Sync + 'static,
    {
        self.pending.insert(0);
        self.task_tx
            .send(manager.source())
            .unwrap();

        self.spawn_workers(Arc::clone(&manager));
        self.process_tasks();
        self.pool.join();
    }

    /* HELPERS */

    fn spawn_workers<M>(&self, manager: Arc<M>)
    where
        M: Manager<B> + Send + Sync + 'static,
    {
        for _ in 0..self.pool.max_count() {
            let rx = self.task_rx.clone();
            let tx = self.task_tx.clone();
            let manager = Arc::clone(&manager);

            self.pool.execute(move || {
                while let Ok(task) = rx.recv() {
                    let new_tasks = match task {
                        Task::Discover(comp, front) => manager
                            .explore(comp, front)
                            .unwrap(),
                        Task::Process(comp, front) => manager
                            .process(comp, front)
                            .unwrap(),
                    };
                    for t in new_tasks {
                        tx.send(t).unwrap();
                    }
                }
            });
        }
    }

    fn process_tasks(&mut self) {
        while let Ok(task) = self.task_rx.recv() {
            let component = task.component();
            match task {
                Task::Discover(comp, front) => {
                    if self.scheduled.insert(comp) {
                        self.pending.insert(comp);
                        self.task_tx
                            .send(Task::Discover(comp, front))
                            .unwrap();
                    }
                },
                Task::Process(comp, front) => {
                    self.store_or_run(comp, front);
                },
            }

            self.completed.insert(component);
            self.pending.remove(&component);
            if self.done() {
                break;
            }
        }
    }

    fn store_or_run(&mut self, comp: Component, front: Frontier<B>) {
        self.accumulated
            .entry(comp)
            .or_default()
            .extend(front);

        if self
            .graph
            .neighbors_directed(comp, Direction::Incoming)
            .all(|dep| self.completed.contains(&dep))
        {
            let combined = self
                .accumulated
                .remove(&comp)
                .unwrap();

            self.task_tx
                .send(Task::Process(comp, combined))
                .unwrap();
        }
    }

    fn done(&self) -> bool {
        self.pending.is_empty()
    }
}
