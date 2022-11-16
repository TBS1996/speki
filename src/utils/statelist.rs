use crate::{
    app::{AppData, Widget},
    MyKey,
};
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders},
    Frame,
};

pub trait KeyHandler {
    // bool represents if the keyhandler used the key. If it didn't use the key, then StaefulList
    // will check if its gonna perform an action with that key instead.
    fn keyhandler(&mut self, _appdata: &AppData, _key: MyKey) -> bool {
        false
    }
}

use super::{
    aliases::Pos,
    libextensions::{MyList, MyListState},
};
use std::fmt::Display;
#[derive(Clone)]
pub struct StatefulList<T: Display + KeyHandler> {
    pub state: MyListState,
    pub items: Vec<T>,
    pub fixed_fields: bool,
    area: Rect,
    pub title: String,
    pub persistent_highlight: bool,
    dragpos: Option<Pos>,
}

impl<T: Display + KeyHandler> StatefulList<T> {
    pub fn with_items<U: Into<String>>(title: U, items: Vec<T>) -> StatefulList<T> {
        let title = title.into();
        let mut thelist = Self {
            state: MyListState::default(),
            items,
            fixed_fields: true,
            area: Rect::default(),
            title,
            persistent_highlight: false,
            dragpos: None,
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
            state: MyListState::default(),
            items,
            fixed_fields: true,
            area: Rect::default(),
            title,
            persistent_highlight: false,
            dragpos: None,
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
    pub fn keypress(&mut self, appdata: &AppData, pos: Pos) {
        if pos.y <= self.area.y
            || pos.x == self.area.x
            || pos.x == self.area.x + self.area.width - 1
            || pos.y == self.area.y + self.area.height - 1
        {
            return;
        }
        let selected_index = pos.y - self.area.y - 1 + self.state.get_offset() as u16;
        if selected_index < (self.items.len()) as u16 {
            self.state.select(Some(selected_index as usize));
            self.keyhandler(appdata, MyKey::Enter);
        }
    }
    fn page_up(&mut self) {
        let height = self.get_area().height - 2;
        if let Some(idx) = self.state.selected() {
            let reloffset = idx - self.state.get_offset();
            if reloffset == 0 {
                let new_index = std::cmp::max(0, reloffset as i32 - height as i32);
                self.state.select(Some(new_index as usize));
            } else {
                self.state.select(Some(idx - reloffset));
            }
        }
    }
    fn page_down(&mut self) {
        let height = self.get_area().height - 2;
        if let Some(idx) = self.state.selected() {
            let reloffset = idx - self.state.get_offset();
            if reloffset == height as usize - 1 {
                let new_index = std::cmp::min(self.items.len() - 1, idx + height as usize);
                self.state.select(Some(new_index));
            } else {
                let new_index = std::cmp::min(
                    self.items.len() - 1,
                    idx + (height as usize - reloffset - 1),
                );
                self.state.select(Some(new_index));
            }
        }
    }

    fn scroll_down(&mut self) {
        if let Some(idx) = self.state.selected() {
            let height = self.get_area().height as usize - 2;
            let reloffset = idx - self.state.get_offset();
            if reloffset == 0 {
                self.next();
            }
            let maxvis = self.state.offset + height - 1;
            if self.items.len() - 1 > maxvis {
                self.state.offset += 1;
            } else {
                self.next();
            }
        }
    }
    fn scroll_up(&mut self) {
        if self.state.offset == 0 {
            self.previous();
            return;
        }
        let height = self.get_area().height - 2;
        if let Some(idx) = self.state.selected() {
            let reloffset = idx - self.state.get_offset();
            if reloffset == height as usize - 1 {
                self.previous();
            }
            self.state.offset -= 1;
        }
    }

    // TODO: find out why theres a buffer overflow instead of copping out with i32
    fn index_from_pos(&mut self, pos: Pos) -> usize {
        let index = (self.state.offset as u16 + pos.y) as i32 - self.get_area().y as i32 - 1;
        std::cmp::max(index, 0) as usize
    }

    fn dragging(&mut self, new: Pos) {
        if !self.is_selected(&new) {
            return;
        }
        if let Some(from_idx) = self.state.selected() {
            let to_index = std::cmp::min(self.index_from_pos(new), self.items.len() - 1);
            self.items.swap(from_idx, to_index);
            self.state.select(Some(to_index));
        }
    }
    fn swap_forward(&mut self) {
        if let Some(idx) = self.state.selected() {
            self.items.swap(idx, idx + 1);
            self.next();
        }
    }

    fn swap_back(&mut self) {
        if let Some(idx) = self.state.selected() {
            self.items.swap(idx, idx - 1);
            self.previous();
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
            MyKey::KeyPress(pos) => self.keypress(appdata, pos),
            MyKey::Char('k') | MyKey::Up => self.previous(),
            MyKey::Char('j') | MyKey::Down => self.next(),
            MyKey::Drag(pos) if !self.fixed_fields => self.dragging(pos),
            MyKey::Char('J') if !self.fixed_fields => self.move_item_down(),
            MyKey::Char('K') if !self.fixed_fields => self.move_item_up(),
            MyKey::Home => self.home_key(),
            MyKey::End => self.end_key(),
            MyKey::ScrollUp => self.scroll_up(),
            MyKey::ScrollDown => self.scroll_down(),
            MyKey::PageUp => self.page_up(),
            MyKey::PageDown => self.page_down(),
            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame<MyType>, _appdata: &AppData, cursor: &Pos) {
        let style = Style::default();
        let area = self.get_area();
        let selected = View::isitselected(area, cursor);
        let title = &self.title;
        let items: Vec<MyListItem> = self
            .items
            .iter()
            .map(|item| {
                let lines = vec![Spans::from(format!("{}", item))];
                MyListItem::new(lines).style(style)
            })
            .collect();

        let bordercolor = if selected { Color::Red } else { Color::White };
        let borderstyle = Style::default().fg(bordercolor);
        let items = MyList::new(items).block(
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

use super::libextensions::MyListItem;
use super::misc::View;
use crate::MyType;

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
