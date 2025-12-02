//! Formatting utilities for dashboard display.

use crate::game::Component;
use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskCategory;
use crate::scheduler::TaskID;

/* IMPLEMENTATIONS */

pub fn format_task_id(category: &TaskCategory, component: Component) -> String {
    format!("{:?}({})", category, component)
}

pub fn format_deps(
    deps: &[TaskID],
    snapshot: &SchedulerSnapshot,
    max: usize,
) -> String {
    if deps.is_empty() {
        return String::new();
    }

    let info: Vec<(TaskCategory, Component)> = deps
        .iter()
        .filter_map(|id| {
            snapshot
                .tasks
                .get(id)
                .map(|ctx| (ctx.category, ctx.component))
        })
        .collect();

    if info.len() <= max {
        format!("Waits on: {}", format_ids(&info))
    } else {
        let shown = &info[..max];
        let more = info.len() - max;
        format!(
            "Waits on: {}, ... (+{} more)",
            format_ids(shown),
            more
        )
    }
}

pub fn format_weight(weight: u64, all: &[u64]) -> Option<String> {
    if all.len() < 2 {
        return None;
    }

    let n = all.len() as f64;
    let sum: u64 = all.iter().sum();
    let mean = sum as f64 / n;

    let var: f64 = all
        .iter()
        .map(|&w| (w as f64 - mean).powi(2))
        .sum::<f64>()
        / n;
    let stddev = var.sqrt();

    if stddev < 0.001 {
        return None;
    }

    let sigma = (weight as f64 - mean) / stddev;
    let sign = if sigma >= 0.0 { "+" } else { "" };
    Some(format!("{}{:.1}σ", sign, sigma))
}

pub fn format_tick(ns: u64) -> String {
    if ns < 1_000 {
        format!("{}ns", ns)
    } else if ns < 1_000_000 {
        format!("{:.1}µs", ns as f64 / 1_000.0)
    } else if ns < 1_000_000_000 {
        format!("{:.2}ms", ns as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", ns as f64 / 1_000_000_000.0)
    }
}

pub fn format_duration(secs: f64) -> String {
    if secs < 60.0 {
        return format!("{:.0}s", secs);
    }

    if secs < 3600.0 {
        return format_minutes(secs);
    }

    if secs < 86400.0 {
        return format_hours(secs);
    }

    format_days(secs)
}

fn format_minutes(secs: f64) -> String {
    let mins = (secs / 60.0) as u64;
    let s = (secs % 60.0) as u64;
    if s > 0 { format!("{}m {}s", mins, s) } else { format!("{}m", mins) }
}

fn format_hours(secs: f64) -> String {
    let hours = (secs / 3600.0) as u64;
    let mins = ((secs % 3600.0) / 60.0) as u64;
    if mins > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}h", hours)
    }
}

fn format_days(secs: f64) -> String {
    let days = (secs / 86400.0) as u64;
    let hours = ((secs % 86400.0) / 3600.0) as u64;
    if hours > 0 {
        format!("{}d {}h", days, hours)
    } else {
        format!("{}d", days)
    }
}

/* HELPERS */

fn format_ids(tasks: &[(TaskCategory, Component)]) -> String {
    tasks
        .iter()
        .map(|(cat, comp)| format_task_id(cat, *comp))
        .collect::<Vec<_>>()
        .join(", ")
}
