
use crate::utils::{sql::fetch::fetch_card, card::Review};
use crate::logic::review::ReviewSelection;
use crate::app::App;
use Direction::{Vertical, Horizontal};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{
        Block, Borders, Gauge},
    Frame,
};





pub fn progress_bar<B>(f: &mut Frame<B>, current: u32, max: u32, color: Color, area: Rect)
where
    B: Backend,

{
    let progress = max - current;
    let percent = (progress as f32 / max as f32) * 100 as f32;
    

    let label = format!("{}/{}", progress, max);
    let gauge = Gauge::default()
        .block(Block::default().title("Progress").borders(Borders::ALL))
        .gauge_style(Style::default().fg(color).bg(Color::Black))
        .percent(percent as u16)
        .label(label);
    f.render_widget(gauge, area); 
}
