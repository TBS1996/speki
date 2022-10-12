use crate::utils::aliases::*;

use crate::utils::statelist::StatefulList;
use crate::widgets::list::StraitList;
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::ListState;
use tui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};

use tui::style::Modifier;
use tui::text::Spans;

#[derive(Clone)]
pub struct CardItem {
    pub question: String,
    pub id: CardID,
}

impl<T> StraitList<T> for StatefulList<CardItem> {
    fn state(&self) -> ListState {
        self.state.clone()
    }

    fn generate_list_items(&self, selected: bool, title: String) -> List {
        let bordercolor = if selected { Color::Red } else { Color::White };
        let style = Style::default().fg(bordercolor);

        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|card| {
                let lines = vec![Spans::from(card.question.clone())];
                ListItem::new(lines).style(Style::default())
            })
            .collect();

        let items = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(style)
                .title(title),
        );

        if selected {
            items.highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            items
        }
    }
}
