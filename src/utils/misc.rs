use crate::logic::review::ReviewMode;
use tui::{style::Color, layout::{Rect, Layout, Direction, Constraint}};






pub fn modecolor(mode: &ReviewMode) -> Color {
    match mode{
        ReviewMode::Review(_) => Color::Red,
        ReviewMode::Unfinished(_) => Color::Yellow,
        ReviewMode::Pending(_) => Color::Cyan,
        ReviewMode::IncRead(_) => Color::Green,
        ReviewMode::Done => Color::Blue,
    }
}

#[derive(Clone)]
pub enum PopUpStatus{
    OnGoing,
    Finished,
}



pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
