pub mod review;
pub mod add_card;
pub mod browse;
pub mod import;
pub mod incread;

use crate::app::App;
use crate::tabs::Widget;
use crate::utils::widgets::find_card::draw_find_card;


use crate::app::PopUp;

use crate::ui::{
    review::main_review,
    add_card::draw_add_card,
    browse::draw_browse,
    incread::draw_incread,
};

use crate::tabs::MyType;
use tui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Tabs},
    Frame,
};
pub fn draw(f: &mut Frame<MyType>, app: &mut App) {
    
    let chunks = Layout::default()
        .constraints(
            [
            Constraint::Length(3), 
            Constraint::Min(0),
            ]
            .as_ref())
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


    match &app.popup{
        PopUp::CardSelecter(cardfinder) => draw_find_card(f, &cardfinder, chunks[1]),
        PopUp::AddChild(addchild) => addchild.render(f,  chunks[1]),
        PopUp::None => {
            f.render_widget(tabs, chunks[0]);
            match app.tabs.index {
                0 => main_review(f,   app, chunks[1]),
                1 => draw_add_card(f, app, chunks[1]),
                2 => draw_incread(f,  app, chunks[1]),
                3 => draw_browse(f,   app, chunks[1]),
                4 => app.debug.render(f, chunks[1]),
                _ => {},
            };
        },
    };
}




