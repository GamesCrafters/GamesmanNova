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

use derive_builder::Builder;

use std::collections::HashMap;

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
        let limit = ctx.units?;
        if running_count < limit {
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
            .keys()
            .map(|tid| (*tid, critical_weight(*tid, ctx.buffer)))
            .min_by_key(|(_, depth)| *depth)?;

        if max_ready_depth > min_running_depth + threshold {
            Some(min_running_tid)
        } else {
            None
        }
    }

    fn execute<'a>(&mut self, ctx: &DecisionContext<'a>) -> Option<TaskID> {
        ctx.candidates
            .keys()
            .map(|tid| (*tid, critical_weight(*tid, ctx.buffer)))
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

    use anyhow::Result;

    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use crate::core::developer::GraphBuilder;
    use crate::core::scheduler::DecisionContext;
    use crate::core::scheduler::Dependencies;
    use crate::core::scheduler::Scheduler;
    use crate::core::scheduler::SchedulerContextBuilder;
    use crate::core::scheduler::SchedulerSnapshot;
    use crate::core::scheduler::SchedulerState;
    use crate::core::scheduler::TaskContextBuilder;
    use crate::core::scheduler::TaskOutcome;
    use crate::core::scheduler::TaskState;
    use crate::core::scheduler::logger::history::HistoryLogger;
    use crate::core::scheduler::logger::history::HistoryLoggerBuilder;
    use crate::core::scheduler::runner::sync::SyncRunner;
    use crate::core::scheduler::task::mock::TaskBuilder;
    use crate::core::scheduler::task::mock::TaskNodeBuilder;
    use crate::traits::scheduler::Logger;
    use crate::traits::scheduler::Policy;
    use crate::traits::scheduler::Runner;

    use super::*;

    /* TEST UTILITIES */

    const MODULE: &str = "critical-policy";

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

    /// Helper to create a scheduler with default test configuration
    fn create_test_scheduler(
        policy: CriticalPathPolicy,
    ) -> Result<(Scheduler, Rc<RefCell<HistoryLogger>>)> {
        let history = HistoryLoggerBuilder::default()
            .frequency(1usize)
            .build()?;

        let (logger, logger_ref) = SharedLogger::new(history);

        let context = SchedulerContextBuilder::default()
            .policy(Box::new(policy) as Box<dyn Policy>)
            .logger(Box::new(logger) as Box<dyn Logger>)
            .runner(Box::new(SyncRunner::default()) as Box<dyn Runner>)
            .build()?;

        let state = SchedulerState::default();
        let scheduler = Scheduler::new(context, state);

        Ok((scheduler, logger_ref))
    }

    /// Helper to find task ID by name in final snapshot
    fn find_task_by_name(
        snapshots: &[SchedulerSnapshot],
        name: &str,
    ) -> TaskID {
        snapshots
            .last()
            .unwrap()
            .tasks
            .iter()
            .find_map(|(tid, ctx)| (ctx.about == name).then_some(*tid))
            .unwrap()
    }

    /* TESTS */

    #[test]
    fn test_executes_by_descending_weight() -> Result<()> {
        // Test multiple ready tasks execute in descending weight order
        // Root discovers A(100), B(50), C(25), D(10)
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

        let c = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("C")
            .size(Some(25))
            .build()?;

        let d = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("D")
            .size(Some(10))
            .build()?;

        let root = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("root")
            .size(Some(5))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&root, &a)
            .edge(&root, &b)
            .edge(&root, &c)
            .edge(&root, &d);

        let task_graph = TaskBuilder::new()
            .name("test-descending-weight")
            .graph(graph)
            .source(&root)
            .build()?;

        task_graph.visualize(MODULE)?;

        let policy = CriticalPathPolicyBuilder::default()
            .sigma(1.0)
            .build()?;

        let (mut scheduler, logger_ref) = create_test_scheduler(policy)?;

        scheduler.register(task_graph.root_task()?)?;
        scheduler.run()?;

        let logger = logger_ref.borrow();
        let snapshots = logger.snapshots();

        let tid_a = find_task_by_name(snapshots, "A");
        let tid_b = find_task_by_name(snapshots, "B");
        let tid_c = find_task_by_name(snapshots, "C");
        let tid_d = find_task_by_name(snapshots, "D");

        // Should execute in order: A(100), B(50), C(25), D(10)
        assert!(
            logger.execution_order(&[tid_a, tid_b, tid_c, tid_d]),
            "Tasks should execute in descending weight order: A, B, C, D"
        );

        Ok(())
    }

    #[test]
    fn test_waits_for_discovered_dependencies() -> Result<()> {
        // Test that tasks properly wait for their discovered children
        // Root -> A -> B (A discovers B dynamically and waits for it)
        let b = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("B")
            .size(Some(50))
            .build()?;

        let a = TaskNodeBuilder::default()
            .ticks(2)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("A")
            .size(Some(100))
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
            .edge(&a, &b);

        let task_graph = TaskBuilder::new()
            .name("test-dependencies")
            .graph(graph)
            .source(&root)
            .build()?;

        task_graph.visualize(MODULE)?;

        let policy = CriticalPathPolicyBuilder::default()
            .sigma(1.0)
            .build()?;

        let (mut scheduler, logger_ref) = create_test_scheduler(policy)?;

        scheduler.register(task_graph.root_task()?)?;
        scheduler.run()?;

        let logger = logger_ref.borrow();
        let snapshots = logger.snapshots();

        let tid_a = find_task_by_name(snapshots, "A");
        let tid_b = find_task_by_name(snapshots, "B");

        // A starts running first, discovers B, then waits for B
        assert!(
            logger.before(tid_a, tid_b),
            "A should start before B (A discovers B dynamically)"
        );

        Ok(())
    }

    #[test]
    fn test_critical_path_weight_propagation() -> Result<()> {
        // Test diamond pattern where critical path weights matter:
        // Root discovers Left(10) and Right(50) simultaneously
        // Left discovers Bottom(100), Right discovers Bottom(100)
        // Critical weights: Left=110, Right=150
        // Right should execute before Left due to higher critical path weight
        let bottom = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("Bottom")
            .size(Some(100))
            .build()?;

        let left = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("Left")
            .size(Some(10))
            .build()?;

        let right = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("Right")
            .size(Some(50))
            .build()?;

        let root = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("Root")
            .size(Some(5))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&root, &left)
            .edge(&root, &right)
            .edge(&left, &bottom)
            .edge(&right, &bottom);

        let task_graph = TaskBuilder::new()
            .name("test-critical-path")
            .graph(graph)
            .source(&root)
            .build()?;

        task_graph.visualize(MODULE)?;

        let policy = CriticalPathPolicyBuilder::default()
            .sigma(1.0)
            .build()?;

        let (mut scheduler, logger_ref) = create_test_scheduler(policy)?;

        scheduler.register(task_graph.root_task()?)?;
        scheduler.run()?;

        let logger = logger_ref.borrow();
        let snapshots = logger.snapshots();

        let tid_left = find_task_by_name(snapshots, "Left");
        let tid_right = find_task_by_name(snapshots, "Right");

        // Right (weight 50, critical path 150) should execute before
        // Left (weight 10, critical path 110)
        assert!(
            logger.before(tid_right, tid_left),
            "Right should execute before Left due to higher critical path weight"
        );

        Ok(())
    }

    #[test]
    fn test_sigma_threshold_prevents_unnecessary_preemption() -> Result<()> {
        let mut policy = CriticalPathPolicyBuilder::default()
            .sigma(1.0)
            .build()?;

        let tid_low = TaskID::from(1u64);
        let tid_high = TaskID::from(2u64);

        // Task A: running, weight 50
        let ctx_low = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Running)
            .size(Some(50))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("low".to_string())
            .progress(None)
            .build()?;

        // Task B: ready, weight 65 (difference = 15)
        // With sufficient stddev, sigma threshold should prevent preemption
        let ctx_high = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Ready)
            .size(Some(65))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("high".to_string())
            .progress(None)
            .build()?;

        // Add more tasks with varied weights to increase stddev
        // Weights: 10, 20, 30, 40, 50, 65 -> mean=35.8, stddev≈19.5
        // Threshold = 19.5, difference = 65-50 = 15 < 19.5 (no preemption)
        let ctx_other1 = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Suspended(TaskOutcome::Success(0)))
            .size(Some(10))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("other1".to_string())
            .progress(None)
            .build()?;

        let ctx_other2 = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Suspended(TaskOutcome::Success(0)))
            .size(Some(20))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("other2".to_string())
            .progress(None)
            .build()?;

        let ctx_other3 = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Suspended(TaskOutcome::Success(0)))
            .size(Some(30))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("other3".to_string())
            .progress(None)
            .build()?;

        let ctx_other4 = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Suspended(TaskOutcome::Success(0)))
            .size(Some(40))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("other4".to_string())
            .progress(None)
            .build()?;

        let mut buffer = HashMap::new();
        buffer.insert(tid_low, ctx_low);
        buffer.insert(tid_high, ctx_high);
        buffer.insert(TaskID::from(3u64), ctx_other1);
        buffer.insert(TaskID::from(4u64), ctx_other2);
        buffer.insert(TaskID::from(5u64), ctx_other3);
        buffer.insert(TaskID::from(6u64), ctx_other4);

        // Candidates for preemption are the running tasks
        let mut candidates = HashMap::new();
        candidates.insert(tid_low, buffer.get(&tid_low).unwrap());

        let ctx = DecisionContext {
            candidates,
            buffer: &buffer,
            units: Some(1),
            ticks: 0,
        };

        // Policy should NOT preempt (weight difference below threshold)
        let decision = policy.preempt(&ctx);
        assert_eq!(
            decision, None,
            "Should not preempt when weight difference below sigma threshold"
        );

        Ok(())
    }

    #[test]
    fn test_preempt_decision_for_higher_weight() -> Result<()> {
        let mut policy = CriticalPathPolicyBuilder::default()
            .sigma(1.0)
            .build()?;

        let tid_low = TaskID::from(1u64);
        let tid_high = TaskID::from(2u64);

        // Task A: running, low weight (size 10)
        let ctx_low = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Running)
            .size(Some(10))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("low".to_string())
            .progress(None)
            .build()?;

        // Task B: ready, high weight (size 100)
        let ctx_high = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Ready)
            .size(Some(100))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("high".to_string())
            .progress(None)
            .build()?;

        let mut buffer = HashMap::new();
        buffer.insert(tid_low, ctx_low);
        buffer.insert(tid_high, ctx_high);

        // Candidates for preemption are the running tasks
        let mut candidates = HashMap::new();
        candidates.insert(tid_low, buffer.get(&tid_low).unwrap());

        let ctx = DecisionContext {
            candidates,
            buffer: &buffer,
            units: Some(1),
            ticks: 0,
        };

        // Policy should decide to preempt the low-weight running task
        let decision = policy.preempt(&ctx);
        assert_eq!(
            decision,
            Some(tid_low),
            "Should preempt low-weight task when higher-weight task is ready"
        );

        Ok(())
    }

    #[test]
    fn test_preempts_lowest_weight_among_multiple_running() -> Result<()> {
        let mut policy = CriticalPathPolicyBuilder::default()
            .sigma(1.0)
            .build()?;

        let tid_low = TaskID::from(1u64);
        let tid_med = TaskID::from(2u64);
        let tid_high = TaskID::from(3u64);
        let tid_ready = TaskID::from(4u64);

        // Three running tasks with different weights
        let ctx_low = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Running)
            .size(Some(10))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("low".to_string())
            .progress(None)
            .build()?;

        let ctx_med = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Running)
            .size(Some(50))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("med".to_string())
            .progress(None)
            .build()?;

        let ctx_high = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Running)
            .size(Some(75))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("high".to_string())
            .progress(None)
            .build()?;

        // One ready task with very high weight
        let ctx_ready = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Ready)
            .size(Some(200))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("ready".to_string())
            .progress(None)
            .build()?;

        let mut buffer = HashMap::new();
        buffer.insert(tid_low, ctx_low);
        buffer.insert(tid_med, ctx_med);
        buffer.insert(tid_high, ctx_high);
        buffer.insert(tid_ready, ctx_ready);

        // All three running tasks are candidates
        let mut candidates = HashMap::new();
        candidates.insert(tid_low, buffer.get(&tid_low).unwrap());

        candidates.insert(tid_med, buffer.get(&tid_med).unwrap());

        candidates.insert(tid_high, buffer.get(&tid_high).unwrap());

        let ctx = DecisionContext {
            candidates,
            buffer: &buffer,
            units: Some(3),
            ticks: 0,
        };

        // Should preempt the lowest weight task (tid_low with weight 10)
        let decision = policy.preempt(&ctx);
        assert_eq!(
            decision,
            Some(tid_low),
            "Should preempt lowest weight task among multiple running tasks"
        );

        Ok(())
    }

    #[test]
    fn test_no_preemption_when_under_capacity() -> Result<()> {
        let mut policy = CriticalPathPolicyBuilder::default()
            .sigma(1.0)
            .build()?;

        let tid_running = TaskID::from(1u64);
        let tid_ready = TaskID::from(2u64);

        let ctx_running = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Running)
            .size(Some(10))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("running".to_string())
            .progress(None)
            .build()?;

        let ctx_ready = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Ready)
            .size(Some(100))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("ready".to_string())
            .progress(None)
            .build()?;

        let mut buffer = HashMap::new();
        buffer.insert(tid_running, ctx_running);
        buffer.insert(tid_ready, ctx_ready);

        let mut candidates = HashMap::new();
        candidates.insert(tid_running, buffer.get(&tid_running).unwrap());

        // Capacity is 2, only 1 running - no preemption needed
        let ctx = DecisionContext {
            candidates,
            buffer: &buffer,
            units: Some(2),
            ticks: 0,
        };

        let decision = policy.preempt(&ctx);
        assert_eq!(
            decision, None,
            "Should not preempt when running count is below capacity"
        );

        Ok(())
    }

    #[test]
    fn test_no_preemption_with_unlimited_capacity() -> Result<()> {
        let mut policy = CriticalPathPolicyBuilder::default()
            .sigma(1.0)
            .build()?;

        let tid_running = TaskID::from(1u64);
        let tid_ready = TaskID::from(2u64);

        let ctx_running = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Running)
            .size(Some(10))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("running".to_string())
            .progress(None)
            .build()?;

        let ctx_ready = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Ready)
            .size(Some(100))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("ready".to_string())
            .progress(None)
            .build()?;

        let mut buffer = HashMap::new();
        buffer.insert(tid_running, ctx_running);
        buffer.insert(tid_ready, ctx_ready);

        let mut candidates = HashMap::new();
        candidates.insert(tid_running, buffer.get(&tid_running).unwrap());

        // Unlimited capacity (None)
        let ctx = DecisionContext {
            candidates,
            buffer: &buffer,
            units: None,
            ticks: 0,
        };

        let decision = policy.preempt(&ctx);
        assert_eq!(
            decision, None,
            "Should not preempt when capacity is unlimited"
        );

        Ok(())
    }

    #[test]
    fn test_no_preemption_when_no_ready_tasks() -> Result<()> {
        let mut policy = CriticalPathPolicyBuilder::default()
            .sigma(1.0)
            .build()?;

        let tid_running = TaskID::from(1u64);

        let ctx_running = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Running)
            .size(Some(10))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("running".to_string())
            .progress(None)
            .build()?;

        let mut buffer = HashMap::new();
        buffer.insert(tid_running, ctx_running);

        let mut candidates = HashMap::new();
        candidates.insert(tid_running, buffer.get(&tid_running).unwrap());

        let ctx = DecisionContext {
            candidates,
            buffer: &buffer,
            units: Some(1),
            ticks: 0,
        };

        let decision = policy.preempt(&ctx);
        assert_eq!(
            decision, None,
            "Should not preempt when no ready tasks exist"
        );

        Ok(())
    }

    #[test]
    fn test_no_preemption_when_all_running_higher_weight() -> Result<()> {
        let mut policy = CriticalPathPolicyBuilder::default()
            .sigma(1.0)
            .build()?;

        let tid_running = TaskID::from(1u64);
        let tid_ready = TaskID::from(2u64);

        // Running task has high weight
        let ctx_running = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Running)
            .size(Some(100))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("running".to_string())
            .progress(None)
            .build()?;

        // Ready task has low weight
        let ctx_ready = TaskContextBuilder::default()
            .executable(None)
            .state(TaskState::Ready)
            .size(Some(10))
            .incoming(Dependencies::new())
            .retriable(false)
            .about("ready".to_string())
            .progress(None)
            .build()?;

        let mut buffer = HashMap::new();
        buffer.insert(tid_running, ctx_running);
        buffer.insert(tid_ready, ctx_ready);

        let mut candidates = HashMap::new();
        candidates.insert(tid_running, buffer.get(&tid_running).unwrap());

        let ctx = DecisionContext {
            candidates,
            buffer: &buffer,
            units: Some(1),
            ticks: 0,
        };

        let decision = policy.preempt(&ctx);
        assert_eq!(
            decision, None,
            "Should not preempt when running tasks have higher weight than ready tasks"
        );

        Ok(())
    }
}
