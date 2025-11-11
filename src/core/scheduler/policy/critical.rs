//! # Critical Path Scheduling Policy
//!
//! Weighted critical path scheduling with preemption support.
//!
//! ## Algorithm
//!
//! Selects tasks with highest critical weight:
//!   weight(task) = size(task) + max(weight(child) for children)
//!
//! This is competitive to ≤ (2 + 1/cores) * optimal under certain
//! assumptions.
//!
//! ## Preemption Strategy
//!
//! Preempts running tasks when ready tasks have significantly
//! higher critical weight. Threshold determined by sigma parameter
//! (difference in standard deviations).
//!
//! ## DecisionContext Usage
//!
//! - execute(): Selects from candidates (Ready or Waiting tasks)
//! - preempt(): Uses candidates (Running) vs buffer (Ready) for
//!   comparison
//! - retry(): Delegates to configurable retry policy closure

use std::collections::HashMap;

use derive_builder::Builder;

use crate::core::scheduler::DecisionContext;
use crate::core::scheduler::SizeStats;
use crate::core::scheduler::TaskID;
use crate::core::scheduler::TaskRegistry;
use crate::traits::scheduler::Policy;

/* TYPE ALIASES */

/// Generic component of a scheduler policy in charge of retrying tasks.
pub type RetryPolicy =
    Box<dyn for<'a> FnMut(&DecisionContext<'a>) -> Option<TaskID>>;

/* STRUCTURES */

/// Weighted critical path scheduling policy with preemption.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct CriticalPathPolicy {
    /// The difference in standard deviations of longest blocked critical path
    /// size among two tasks that will cause the one blocking the lighter path
    /// to be immediately preempted (without necessarily scheduling the other).
    #[builder(default = "1.0f64")]
    pub sigma: f64,

    /// Custom retry policy, determining which tasks the scheduler will mark as
    /// ready for execution (immediately and regardless of their state).
    #[builder(default = "threshold(0)")]
    pub retry: RetryPolicy,
}

/* IMPLEMENTATIONS */

impl Policy for CriticalPathPolicy {
    fn retry<'a>(&mut self, ctx: &DecisionContext<'a>) -> Option<TaskID> {
        (self.retry)(ctx)
    }

    fn preempt<'a>(&mut self, ctx: &DecisionContext<'a>) -> Option<TaskID> {
        let running_count = ctx.candidates.len();
        if ctx.units == 0 || running_count < ctx.units {
            return None;
        }

        let stats = compute_size_stats(ctx.buffer);
        let threshold = (self.sigma * stats.stddev) as u64;

        // Find max critical weight among ready tasks (need to check buffer)
        let max_ready_depth = ctx
            .buffer
            .iter()
            .filter(|(_, task_ctx)| {
                matches!(
                    task_ctx.state,
                    crate::core::scheduler::TaskState::Ready
                )
            })
            .map(|(tid, _)| critical_weight(*tid, ctx.buffer))
            .max()?;

        // Find minimum critical weight among running tasks (candidates)
        let (min_running_tid, min_running_depth) = ctx
            .candidates
            .iter()
            .map(|(tid, _ctx)| (*tid, critical_weight(*tid, ctx.buffer)))
            .min_by_key(|(_, depth)| *depth)?;

        if max_ready_depth > min_running_depth + threshold {
            Some(min_running_tid)
        } else {
            None
        }
    }

    fn execute<'a>(&mut self, ctx: &DecisionContext<'a>) -> Option<TaskID> {
        ctx.candidates
            .iter()
            .map(|(tid, _ctx)| (*tid, critical_weight(*tid, ctx.buffer)))
            .max_by_key(|(_, depth)| *depth)
            .map(|(tid, _)| tid)
    }
}

/* FUNCTIONS */

/// Provide each task up to `limit` retry opportunities.
pub fn threshold(limit: usize) -> RetryPolicy {
    let mut counts: HashMap<TaskID, usize> = HashMap::new();
    let policy = move |ctx: &DecisionContext| {
        let tid = *ctx.candidates.keys().next()?;

        let count = counts.entry(tid).or_insert(0);
        if *count < limit {
            *count += 1;
            Some(tid)
        } else {
            None
        }
    };

    Box::new(policy)
}

/* HELPER FUNCTIONS */

/// Returns size(task) + max{critical_path(child) : child depends on task}.
/// Scheduling this way is competitive to <= (2 + 1/cores) * optimal. (Under
/// certain unrealistic but pretty well-posed assumptions.)
fn critical_weight(tid: TaskID, registry: &TaskRegistry) -> u64 {
    let avg_size = compute_size_stats(registry).mean as u64;
    let mut memo = HashMap::new();
    let mut stack = vec![(tid, false)];

    while let Some((current, children_pushed)) = stack.pop() {
        if memo.contains_key(&current) {
            continue;
        }

        let Some(ctx) = registry.get(&current) else {
            memo.insert(current, 0);
            continue;
        };

        if !children_pushed {
            stack.push((current, true));
            for child in &ctx.incoming {
                if !memo.contains_key(child) {
                    stack.push((*child, false));
                }
            }
        } else {
            let task_size = ctx.size.unwrap_or(avg_size);
            let max_child = ctx
                .incoming
                .iter()
                .filter_map(|child| memo.get(child))
                .max()
                .copied()
                .unwrap_or(0);

            memo.insert(current, task_size + max_child);
        }
    }

    memo.get(&tid)
        .copied()
        .unwrap_or(0)
}

/// Compute task size statistics across the registry.
fn compute_size_stats(registry: &TaskRegistry) -> SizeStats {
    let sizes: Vec<u64> = registry
        .values()
        .filter_map(|ctx| ctx.size)
        .collect();

    if sizes.is_empty() {
        return SizeStats {
            stddev: 0.0,
            mean: 1.0,
        };
    }

    let mean = sizes.iter().sum::<u64>() as f64 / sizes.len() as f64;
    let variance = sizes
        .iter()
        .map(|&s| {
            let diff = s as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / sizes.len() as f64;

    SizeStats {
        stddev: variance.sqrt(),
        mean,
    }
}

/* TESTS */

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use anyhow::Result;

    use super::*;
    use crate::core::developer::GraphBuilder;
    use crate::core::scheduler::Scheduler;
    use crate::core::scheduler::SchedulerContextBuilder;
    use crate::core::scheduler::SchedulerSnapshot;
    use crate::core::scheduler::SchedulerState;
    use crate::core::scheduler::TaskOutcome;
    use crate::core::scheduler::logger::history::HistoryLogger;
    use crate::core::scheduler::logger::history::HistoryLoggerBuilder;
    use crate::core::scheduler::runner::sync::SyncRunner;
    use crate::core::scheduler::task::mock::TaskBuilder;
    use crate::core::scheduler::task::mock::TaskNodeBuilder;
    use crate::traits::scheduler::Logger;
    use crate::traits::scheduler::Policy;
    use crate::traits::scheduler::Runner;

    /* TEST UTILITIES */

    /// Wrapper for HistoryLogger that allows shared access in tests
    struct SharedLogger {
        inner: Rc<RefCell<HistoryLogger>>,
    }

    impl SharedLogger {
        fn new(logger: HistoryLogger) -> (Self, Rc<RefCell<HistoryLogger>>) {
            let inner = Rc::new(RefCell::new(logger));
            let shared = SharedLogger {
                inner: inner.clone(),
            };
            (shared, inner)
        }
    }

    impl Logger for SharedLogger {
        fn observe(
            &mut self,
            snapshot: &SchedulerSnapshot,
            changed: bool,
        ) -> Result<()> {
            self.inner
                .borrow_mut()
                .observe(snapshot, changed)
        }
    }

    /* TESTS */

    #[test]
    fn test_debug_scheduler_execution() -> Result<()> {
        // Debug: step through scheduler execution
        let a = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("A")
            .size(Some(100))
            .build()?;

        let b = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("B")
            .size(Some(50))
            .build()?;

        let root = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("root")
            .size(Some(10))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&root, &a)
            .edge(&root, &b);

        let task_graph = TaskBuilder::new()
            .name("test-debug-exec")
            .graph(graph)
            .source(&root)
            .build()?;

        let root_task = task_graph.root_task()?;
        println!("Root task TID: {}", root_task.tid);
        println!("Root task requires: {:?}", root_task.requires);

        // Set up scheduler
        let history = HistoryLoggerBuilder::default()
            .frequency(1usize)
            .build()?;
        let (logger, _) = SharedLogger::new(history);

        let policy = CriticalPathPolicyBuilder::default()
            .sigma(1.0)
            .build()?;

        let context = SchedulerContextBuilder::default()
            .policy(Box::new(policy) as Box<dyn Policy>)
            .logger(Box::new(logger) as Box<dyn Logger>)
            .runner(Box::new(SyncRunner::default()) as Box<dyn Runner>)
            .build()?;

        let state = SchedulerState::default();
        let mut scheduler = Scheduler::new(context, state);

        println!("Registering root task...");
        scheduler.register(root_task)?;
        println!("Root registered successfully");

        println!("Running scheduler...");
        scheduler.run()?;
        println!("Scheduler completed");

        Ok(())
    }

    #[test]
    fn test_debug_graph_structure() -> Result<()> {
        // Debug: understand what dependencies root has
        let a = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let b = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let root = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&root, &a)
            .edge(&root, &b);

        let task_graph = TaskBuilder::new()
            .name("test-debug")
            .graph(graph)
            .source(&root)
            .build()?;

        // Build the root task and inspect it
        let root_task = task_graph.root_task()?;
        println!("Root task TID: {}", root_task.tid);
        println!("Root task requires: {:?}", root_task.requires);
        println!(
            "Root task requires len: {}",
            root_task.requires.len()
        );

        assert_eq!(
            root_task.requires.len(),
            0,
            "Root should have no requires"
        );

        Ok(())
    }

    #[test]
    fn test_execute_selects_highest_critical_weight() -> Result<()> {
        // Test with root discovering two children with different sizes
        // Root -> A(100), B(50)
        // A should execute before B due to higher critical weight
        let a = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("A")
            .size(Some(100))
            .build()?;

        let b = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("B")
            .size(Some(50))
            .build()?;

        let root = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("root")
            .size(Some(10))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&root, &a)
            .edge(&root, &b);

        let task_graph = TaskBuilder::new()
            .name("test-multi")
            .graph(graph)
            .source(&root)
            .build()?;

        // Set up scheduler
        let history = HistoryLoggerBuilder::default()
            .frequency(1usize)
            .build()?;

        let (logger, logger_ref) = SharedLogger::new(history);
        let policy = CriticalPathPolicyBuilder::default()
            .sigma(1.0)
            .build()?;

        let context = SchedulerContextBuilder::default()
            .policy(Box::new(policy) as Box<dyn Policy>)
            .logger(Box::new(logger) as Box<dyn Logger>)
            .runner(Box::new(SyncRunner::default()) as Box<dyn Runner>)
            .build()?;

        let state = SchedulerState::default();
        let mut scheduler = Scheduler::new(context, state);

        // Register and run
        scheduler.register(task_graph.root_task()?)?;
        scheduler.run()?;

        // Verify execution order
        let logger = logger_ref.borrow();
        let snapshots = logger.snapshots();

        // Verify we got snapshots and all tasks completed
        assert!(
            !snapshots.is_empty(),
            "Should have recorded snapshots"
        );

        let final_snapshot = snapshots.last().unwrap();
        assert!(
            final_snapshot.tasks.len() >= 3,
            "Should have root + 2 children = 3 tasks, got {}",
            final_snapshot.tasks.len()
        );

        // Find task IDs by size
        let tasks: Vec<_> = final_snapshot
            .tasks
            .iter()
            .filter(|(_, ctx)| ctx.size.is_some())
            .collect();

        let tid_100 = *tasks
            .iter()
            .find(|(_, ctx)| ctx.size == Some(100))
            .expect("Should find task with size 100")
            .0;

        let tid_50 = *tasks
            .iter()
            .find(|(_, ctx)| ctx.size == Some(50))
            .expect("Should find task with size 50")
            .0;

        // Verify A (size 100) executed before B (size 50)
        assert!(
            logger.before(tid_100, tid_50),
            "Task with size 100 should run before task with size 50"
        );

        Ok(())
    }
}
