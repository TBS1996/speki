use crate::{
    app::{AppData, Widget},
    MyKey,
};
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
    fn keyhandler(&mut self, _appdata: &AppData, _key: MyKey) -> bool {
        false
    }
}

use std::fmt::Display;

#[derive(Clone)]
pub struct StatefulList<T: Display + KeyHandler> {
    pub state: ListState,
    pub items: Vec<T>,
    fixed_fields: bool,
    area: Rect,
    pub title: String,
    pub persistent_highlight: bool,
}

impl<T: Display + KeyHandler> StatefulList<T> {
    pub fn with_items(title: String, items: Vec<T>) -> StatefulList<T> {
        let mut thelist = Self {
            state: ListState::default(),
            items,
            fixed_fields: true,
            area: Rect::default(),
            title,
            persistent_highlight: false,
        };
        thelist.next();
        thelist
    }

    pub fn with_generic<W, U>(title: String, input: Vec<W>, transformer: U) -> StatefulList<T>
    where
        U: FnMut(W) -> T,
    {
        let generic_vec = input.into_iter().map(transformer).collect();
        Self::with_items(title, generic_vec)
    }

    pub fn new(title: String) -> StatefulList<T> {
        let items = Vec::<T>::new();
        StatefulList {
            state: ListState::default(),
            items,
            fixed_fields: true,
            area: Rect::default(),
            title,
            persistent_highlight: false,
        }
    }

    pub fn replace_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.state.select(None);
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

    pub fn push(&mut self, item: T) {
        self.items.push(item);
        if self.state.selected().is_none() {
            self.state.select(Some(0));
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

    fn home_key(&mut self) {
        if !self.items.is_empty() {
            self.state.select(Some(0));
        }
    }

    fn end_key(&mut self) {
        if !self.items.is_empty() {
            let qty = self.items.len();
            self.state.select(Some(qty - 1));
        }
    }
}

impl<T: Display + KeyHandler> Widget for StatefulList<T> {
    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
    fn get_area(&self) -> Rect {
        self.area
    }

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        if let Some(idx) = self.state.selected() {
            if self.items[idx].keyhandler(appdata, key.clone()) {
                return;
            }
        }

        match key {
            MyKey::Char('k') | MyKey::Up => self.previous(),
            MyKey::Char('j') | MyKey::Down => self.next(),
            MyKey::Char('J') if !self.fixed_fields => self.move_item_down(),
            MyKey::Char('K') if !self.fixed_fields => self.move_item_up(),
            MyKey::Home => self.home_key(),
            MyKey::End => self.end_key(),
            MyKey::ScrollUp => self.previous(),
            MyKey::ScrollDown => self.next(),
            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame<MyType>, _appdata: &AppData, cursor: &(u16, u16)) {
        let style = Style::default();
        let area = self.get_area();
        let selected = View::isitselected(area, cursor);
        let title = &self.title;
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
                .title(title.to_owned()),
        );

        let items = if selected || self.persistent_highlight {
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

use super::misc::View;

#[derive(Clone)]
pub struct TextItem {
    text: String,
}

impl TextItem {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl KeyHandler for TextItem {}
impl Display for TextItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}
