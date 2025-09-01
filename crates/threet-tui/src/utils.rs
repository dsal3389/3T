use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;

/// returns the middle area of the given `Rect` based
/// on the requested (x, y) dem
pub fn get_middle_area(dem: (u16, u16), area: Rect) -> Rect {
    let [_, middle, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(dem.0),
        Constraint::Fill(1),
    ])
    .areas(area);
    let [_, middle, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Max(dem.1),
        Constraint::Fill(1),
    ])
    .areas(middle);
    middle
}
