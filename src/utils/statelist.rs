use crate::utils::sql::fetch::load_cards;
use rusqlite::Connection;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct StatefulList<T: Display> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T: Display> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }
    pub fn new() -> StatefulList<T> {
        let items = Vec::<T>::new();
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn load_cards(conn: &Arc<Mutex<Connection>>) -> StatefulList<u32> {
        let cardvec = load_cards(&conn).unwrap();
        let mut items = Vec::<u32>::new();
        for card in cardvec {
            items.push(card.id);
        }

        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn load_empty() -> StatefulList<u32> {
        let items = Vec::<u32>::new();
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn move_item_up(&mut self) {
        if let Some(idx) = self.state.selected() {
            if idx != 0 {
                self.items.swap(idx, idx - 1);
            }
        }
    }
    pub fn move_item_down(&mut self) {
        if let Some(idx) = self.state.selected() {
            if idx != self.items.len() - 1 {
                self.items.swap(idx, idx + 1);
            }
        }
    }

    pub fn next(&mut self) {
        if self.items.len() == 0 {
            return;
        };

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    self.items.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.len() == 0 {
            return;
        };

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn render(
        &mut self,
        f: &mut Frame<MyType>,
        area: Rect,
        selected: bool,
        title: &str,
        style: Style,
    ) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| {
                let lines = vec![Spans::from(format!("{}", item))];
                ListItem::new(lines).style(style)
            })
            .collect();

        let bordercolor = if selected { Color::Red } else { Color::White };
        let borderstyle = Style::default().fg(bordercolor);
        let items = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(borderstyle)
                .title(title),
        );

        let items = if selected {
            items.highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            items
        };

        f.render_stateful_widget(items, area, &mut self.state);
    }
}

impl<T: Clone + Display> StatefulList<T> {
    pub fn clone_selected(&mut self) -> Option<T> {
        if let Some(index) = self.state.selected() {
            return Some(self.items[index].clone());
        }
        None
    }
}

impl<T: Copy + Display> StatefulList<T> {
    pub fn copy_selected(&mut self) -> Option<T> {
        if let Some(index) = self.state.selected() {
            return Some(self.items[index]);
        }
        None
    }
}

use crate::MyType;
