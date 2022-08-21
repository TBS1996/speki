
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





pub fn view_dependencies<B>(f: &mut Frame<B>, id: u32, app: & App, area: Rect, selected: bool)
where
    B: Backend,
{
    let thecard = fetch_card(&app.conn, id);
    let dep_ids = &thecard.dependencies;
    
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);

   
    let mut deps = Vec::<Row>::new();
    for dep_id in dep_ids{
        let dep = fetch_card(&app.conn, *dep_id);
        let ques = dep.question.clone();
        let foo = Row::new(vec![Cell::from(Span::raw(ques))]);
        deps.push(foo.clone());
    }
    
    
    let table = Table::new(deps).block(Block::default().title("dependencies").borders(Borders::ALL).border_style(style)).widths(&[
            Constraint::Ratio(1, 1),
        ]);
    f.render_widget(table, area);
}
