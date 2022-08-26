
use crate::utils::sql::fetch::fetch_card;
use crate::app::App;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{
        Block, Borders, Row, Table, Cell},
    Frame,
};




pub fn card_status<B>(f: &mut Frame<B>, _app: & App, area: Rect, selected: bool)
where
    B: Backend,
{

    if _app.review.review_cards.is_empty(){return}
    let card_id = _app.review.review_cards[0];
    let card = fetch_card(&_app.conn, card_id);

    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);

    
    let rows = vec![
        Row::new(vec![Cell::from(Span::raw(format!("strength: {}, stability: {}, initiated: {:?}, complete: {:?}, resolved: {:?}, suspended: {:?}", card.strength, card.stability, card.status.complete, card.status.resolved, card.status.suspended, card.status.initiated)))]),
    ];

    
    let table = Table::new(rows).block(Block::default().title("stats").borders(Borders::ALL).border_style(style)).widths(&[
            Constraint::Ratio(1, 1),
        ]);
    f.render_widget(table, area);

}
