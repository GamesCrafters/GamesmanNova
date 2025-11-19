//! Layout calculation for dashboard sections.

use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;

use super::SectionConfig;
use super::TaskFilter;

/* CONSTANTS */

pub const HEADER_HEIGHT: u16 = 5;
pub const MIN_SECTION_HEIGHT: u16 = 5;
pub const LINES_PER_TASK: usize = 3;
pub const OVERFLOW_LINES: usize = 1;

/* LAYOUT */

pub fn layout(area: Rect, sections: &[SectionConfig]) -> std::rc::Rc<[Rect]> {
    let total: usize = sections
        .iter()
        .map(|s| s.weight)
        .sum();

    let available = area
        .height
        .saturating_sub(HEADER_HEIGHT) as usize;
    let mut constraints = vec![Constraint::Length(HEADER_HEIGHT)];

    for (i, section) in sections.iter().enumerate() {
        let proportion =
            (section.weight as f64 / total as f64 * available as f64) as u16;
        let height = proportion.max(MIN_SECTION_HEIGHT);

        if i == sections.len() - 1 {
            constraints.push(Constraint::Min(height));
        } else {
            constraints.push(Constraint::Length(height));
        }
    }

    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
}

pub fn section_shows_time(filters: &[TaskFilter]) -> bool {
    filters.iter().any(|f| {
        matches!(
            f,
            TaskFilter::Running | TaskFilter::Preempting | TaskFilter::Ready
        )
    })
}
