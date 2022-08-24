
use crate::utils::sql::fetch::fetch_card;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, ListItem, List},
    Frame,
};

use crate::app::App;
use crate::logic::browse::BrowseCursor;

pub fn draw_browse<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2),Constraint::Ratio(1, 2),].as_ref(),)
        .split(area);


    filtered_cards(f, _app, chunks[0]);
    selected_cards(f, _app, chunks[1])

}



fn filtered_cards<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{
    let items: Vec<ListItem> = _app.browse.filtered.items.iter().map(|id| {
        let lines = vec![Spans::from(fetch_card(&_app.conn, *id).question.clone())];
        ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::Cyan))
    }).collect();
    
    let mut items = List::new(items).block(Block::default().borders(Borders::ALL).title("Filtered"));

    if let BrowseCursor::Filtered = _app.browse.cursor{
        items = items
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">>> ");
        
    }
    f.render_stateful_widget(items, area, &mut _app.browse.filtered.state);

}


fn selected_cards<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{

    let items: Vec<ListItem> = _app.browse.selected.items.iter().map(|id| {
        let lines = vec![Spans::from(fetch_card(&_app.conn, *id).question)];
        ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::Red))
    }).collect();
    


    
    let mut items = List::new(items).block(Block::default().borders(Borders::ALL).title("Selected"));


    if let BrowseCursor::Selected = _app.browse.cursor{
        items = items
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">>> ");
        
    }

    f.render_stateful_widget(items, area, &mut _app.browse.selected.state);

}
