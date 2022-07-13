pub mod review;
pub mod add_card;
pub mod browse;
pub mod mywidgets;

use crate::app::App;


use crate::ui::{
    review::draw_review,
    add_card::draw_add_card,
    browse::draw_browse,
};


use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Tabs},
    Frame,
};
pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());
    let titles = app
        .tabs
        .titles
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.index);
    f.render_widget(tabs, chunks[0]);
    match app.tabs.index {
        0 => draw_review(f, app, chunks[1]),
        1 => draw_add_card(f, app, chunks[1]),
        2 => draw_browse(f, app, chunks[1]),
        _ => {}
    };

}
