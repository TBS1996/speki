
use crate::utils::sql::fetch::fetch_card;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    widgets::{
        Block, Borders},
    Frame,
};


#[derive(Clone, PartialEq)]
pub struct Dependent{
    pub question: String,
    pub id: u32,
}



use std::sync::{Arc, Mutex};


use rusqlite::Connection;
use crate::utils::widgets::list::list_widget;
use tui::text::Spans;
use tui::style::Modifier;
use crate::utils::aliases::*;

pub fn view_dependents<B>(f: &mut Frame<B>, id: CardID, conn: &Arc<Mutex<Connection>>, area: Rect, selected: bool)
where
    B: Backend,
{
    let thecard = fetch_card(&conn, id);
    let dep_ids = &thecard.dependents;
    let mut dependency_vec = Vec::<Dependent>::new();

    for id in dep_ids{
        dependency_vec.push(
            Dependent{
                question: fetch_card(&conn, *id).question,
                id: *id,
            }
        );
    }
    let statelist = StatefulList::with_items(dependency_vec);
    list_widget(f, &statelist, area, selected, "Dependents".to_string())
}


use crate::utils::statelist::StatefulList;
use crate::utils::widgets::list::StraitList;
use tui::widgets::ListItem;
use tui::widgets::ListState;
use tui::widgets::List;

impl<T> StraitList<T> for StatefulList<Dependent>{

    fn state(&self) -> ListState {
        self.state.clone()
    }

    fn generate_list_items(&self, selected: bool, title: String) -> List{
    let bordercolor = if selected {Color::Red} else {Color::White};
    let style = Style::default().fg(bordercolor);

    let items: Vec<ListItem> = self.items.iter().map(|dependent| {
        let lines = vec![Spans::from(dependent.question.clone())];
        ListItem::new(lines).style(Style::default())
    }).collect();
    
    let items = List::new(items)
        .block(
            Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title(title)
            );
    
    if selected{
    items
        .highlight_style(
            Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )}
    else {items}
    }
}
