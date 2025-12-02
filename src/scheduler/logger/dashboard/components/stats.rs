//! Stats display component.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::scheduler::RunnerSnapshot;
use crate::scheduler::SchedulerSnapshot;

use super::super::support::state::count_states;

/* IMPLEMENTATIONS */

pub fn render(
    frame: &mut Frame,
    area: Rect,
    snapshot: &SchedulerSnapshot,
    runner: Option<&RunnerSnapshot>,
) {
    let counts = count_states(snapshot);

    let workers = runner
        .map(|r| {
            let cap = r
                .capacity
                .map(|c| c.to_string())
                .unwrap_or_else(|| "∞".to_string());
            format!("{}/{}", counts.running, cap)
        })
        .unwrap_or_else(|| format!("{}", counts.running));

    let title = format!(
        " Tick: {} │ Workers: {} │ Tasks: {} ",
        snapshot.tick,
        workers,
        snapshot.tasks.len()
    );

    let lines = vec![Line::from(title)];

    frame.render_widget(Paragraph::new(lines), area);
}
