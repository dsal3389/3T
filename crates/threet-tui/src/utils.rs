use ratatui::layout::{Constraint, Layout, Rect};

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
