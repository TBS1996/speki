
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





pub fn view_dependents<B>(f: &mut Frame<B>, id: u32, app: & App, area: Rect, selected: bool)
where
    B: Backend,
{
    let thecard = fetch_card(&app.conn, id);
    let dep_ids = &thecard.dependents;
    

   
    let mut deps = Vec::<Row>::new();
    for dep_id in dep_ids{
        let dep = fetch_card(&app.conn, *dep_id);
        let foo = Row::new(vec![Cell::from(Span::raw(dep.question.clone()))]);
        deps.push(foo);
    }
    
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);
    
    let table = Table::new(deps)
        .block(
            Block::default()
                .title("dependents")
                .borders(Borders::ALL)
                .border_style(style))
        .widths(&[
            Constraint::Ratio(1, 1),
        ]);
    f.render_widget(table, area);
}
