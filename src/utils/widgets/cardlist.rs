use crate::utils::aliases::*;

use crate::utils::statelist::StatefulList;
use crate::utils::widgets::list::StraitList;
use tui::widgets::ListItem;
use tui::widgets::ListState;
use tui::widgets::List;
use tui::{
    style::{Color, Style},
    widgets::{
        Block, Borders},
};


use std::sync::{Arc, Mutex};

use tui::text::Spans;
use tui::style::Modifier;


#[derive(Clone)]
pub struct CardItem{
    pub question: String,
    pub id: CardID,
}



/*
pub fn view_cards<B>(f: &mut Frame<B>, id: u32, conn: &Arc<Mutex<Connection>>, area: Rect, selected: bool)
where
    B: Backend,
{
    let thecard = fetch_card(conn, id);
    let mut cardvec = Vec::<CardItem>::new();

    for id in dep_ids{
        cardvec.push(
            CardItem{
                question: fetch_card(conn, *id).question,
                id: *id,
            }
        );
    }
    let statelist = StatefulList::with_items(cardvec);
    list_widget(f, &statelist, area, selected)
}
*/

impl<T> StraitList<T> for StatefulList<CardItem>{

    fn state(&self) -> ListState {
        self.state.clone()
    }

    fn generate_list_items(&self, selected: bool, title: String) -> List{
        let bordercolor = if selected {Color::Red} else {Color::White};
        let style = Style::default().fg(bordercolor);

        let items: Vec<ListItem> = self.items.iter().map(|card| {
            let lines = vec![Spans::from(card.question.clone())];
            ListItem::new(lines)
                .style(Style::default())})
            .collect();
        
        let items = List::new(items)
            .block(Block::default()
                   .borders(Borders::ALL)
                   .border_style(style)
                   .title(title));
        
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
