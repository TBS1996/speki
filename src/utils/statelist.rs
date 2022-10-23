use crate::MyKey;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub trait KeyHandler {
    // bool represents if the keyhandler used the key. If it didn't use the key, then StaefulList
    // will check if its gonna perform an action with that key instead.
    fn keyhandler(&mut self, _key: MyKey) -> bool {
        false
    }
}

use std::fmt::Display;

#[derive(Clone)]
pub struct StatefulList<T: Display + KeyHandler> {
    pub state: ListState,
    pub items: Vec<T>,
    fixed_fields: bool,
}

impl<T: Display + KeyHandler> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
            fixed_fields: false,
        }
    }
    pub fn new() -> StatefulList<T> {
        let items = Vec::<T>::new();
        StatefulList {
            state: ListState::default(),
            items,
            fixed_fields: false,
        }
    }

    pub fn move_item_up(&mut self) {
        if let Some(idx) = self.state.selected() {
            if idx != 0 {
                self.items.swap(idx, idx - 1);
                self.previous();
            }
        }
    }
    pub fn move_item_down(&mut self) {
        if let Some(idx) = self.state.selected() {
            if idx != self.items.len() - 1 {
                self.items.swap(idx, idx + 1);
                self.next();
            }
        }
    }

    pub fn take_selected_item(&mut self) -> Option<T> {
        if let Some(idx) = self.state.selected() {
            if idx == self.items.len() - 1 {
                self.previous();
            }
            if self.items.len() == 1 {
                self.state.select(None);
            }
            Some(self.items.remove(idx))
        } else {
            None
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
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
        if self.items.is_empty() {
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

    pub fn keyhandler(&mut self, key: MyKey) {
        if let Some(idx) = self.state.selected() {
            if self.items[idx].keyhandler(key.clone()) {
                return;
            }
        }

        match key {
            MyKey::Char('k') | MyKey::Up => self.previous(),
            MyKey::Char('j') | MyKey::Down => self.next(),
            MyKey::Char('J') if !self.fixed_fields => self.move_item_down(),
            MyKey::Char('K') if !self.fixed_fields => self.move_item_up(),
            _ => {}
        }
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

impl<T: Clone + Display + KeyHandler> StatefulList<T> {
    pub fn clone_selected(&mut self) -> Option<T> {
        if let Some(index) = self.state.selected() {
            return Some(self.items[index].clone());
        }
        None
    }
}

impl<T: Copy + Display + KeyHandler> StatefulList<T> {
    pub fn copy_selected(&mut self) -> Option<T> {
        if let Some(index) = self.state.selected() {
            return Some(self.items[index]);
        }
        None
    }
}

use crate::MyType;
