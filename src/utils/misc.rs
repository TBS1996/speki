use crate::logic::review::ReviewMode;
use tui::style::Color;






pub fn modecolor(mode: &ReviewMode) -> Color {
    match mode{
        ReviewMode::Review(_) => Color::Red,
        ReviewMode::Unfinished(_) => Color::Yellow,
        ReviewMode::Pending(_) => Color::Cyan,
        ReviewMode::IncRead(_) => Color::Green,
        ReviewMode::Done => Color::Blue,
    }
}

pub enum PopUpStatus{
    OnGoing,
    Finished,
}
