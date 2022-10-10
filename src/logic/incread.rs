use crate::utils::sql::update::update_inc_text;
use crate::Direction;
use crate::MyKey;

use crate::utils::aliases::*;
use crate::utils::statelist::StatefulList;
use crate::utils::widgets::list::StraitList;
use crate::utils::widgets::topics::TopicList;
use tui::widgets::ListState;

use tui::{
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem},
};

use crate::utils::incread::IncListItem;
use crate::utils::incread::IncRead;
use crate::utils::widgets::textinput::Field;
use std::sync::{Arc, Mutex};

#[derive(PartialEq)]
pub enum Selection {
    Incread,
    List,
    Extracts,
    Topics,
}

pub enum Menu {
    Main,
    WikiSelect(WikiSelect),
}

pub struct WikiSelect {
    pub searchbar: Field,
}

impl WikiSelect {
    fn new() -> Self {
        WikiSelect {
            searchbar: Field::new(),
        }
    }
}

pub struct MainInc {
    pub inclist: StatefulList<IncListItem>,
    pub focused: Option<IncRead>,
    pub selection: Selection,
    pub extracts: StatefulList<IncListItem>,
    pub topics: TopicList,
    pub menu: Menu,
}

use crate::utils::sql::fetch::load_extracts;
use crate::utils::sql::fetch::{get_incread, load_inc_items};
use crate::utils::sql::insert::new_incread;
use rusqlite::Connection;

impl MainInc {
    pub fn new(conn: &Arc<Mutex<Connection>>) -> Self {
        let items = load_inc_items(conn, 1).unwrap();
        let foo = StatefulList::with_items(items);
        let mut topics = TopicList::new(conn);
        topics.next();
        let focused: Option<IncRead> = None;
        let menu = Menu::Main;
        MainInc {
            inclist: foo,
            focused,
            selection: Selection::List,
            extracts: StatefulList::<IncListItem>::new(),
            topics,
            menu,
        }
    }

    pub fn update_text(&self, conn: &Arc<Mutex<Connection>>) {
        if let Some(inc) = &self.focused {
            let id = inc.id;
            let text = inc.source.return_text();
            update_inc_text(conn, text, id, &inc.source.cursor).unwrap();
        }
    }

    pub fn create_source(&mut self, conn: &Arc<Mutex<Connection>>, text: String) {
        let topic: TopicID = self.topics.get_selected_id().unwrap();
        new_incread(conn, 0, topic, text, true).unwrap();
        self.reload_inc_list(conn);
    }

    pub fn new_focus(&mut self, conn: &Arc<Mutex<Connection>>) {
        if let Selection::List = self.selection {
            if let Some(idx) = self.inclist.state.selected() {
                let id: IncID = self.inclist.items[idx].id;
                let foo = get_incread(conn, id).unwrap();
                self.focused = Some(foo);
                self.extracts.items = load_extracts(conn, id).unwrap();
            }
        } else if let Selection::Extracts = self.selection {
            if let Some(idx) = self.extracts.state.selected() {
                let id: IncID = self.extracts.items[idx].id;
                self.focused = Some(get_incread(conn, id).unwrap());
                self.extracts.items = load_extracts(conn, id).unwrap();
            }
        }
    }

    pub fn reload_inc_list(&mut self, conn: &Arc<Mutex<Connection>>) {
        let items = load_inc_items(conn, self.topics.get_selected_id().unwrap()).unwrap();
        let foo = StatefulList::with_items(items);
        self.inclist = foo;
    }

    pub fn reload_extracts(&mut self, conn: &Arc<Mutex<Connection>>, id: IncID) {
        self.extracts.items = load_extracts(conn, id).unwrap();
    }

    pub fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey) {
        use crate::MyKey::*;
        use Selection::*;
        if let MyKey::Nav(dir) = &key {
            self.nav_inc(conn, dir);
            return;
        } else if let MyKey::Alt('a') = &key {
            self.create_source(conn, "".to_string());
            return;
        } else if let Menu::WikiSelect(wiki) = &mut self.menu {
            match key {
                Esc => self.menu = Menu::Main,
                Enter => {
                    let text = wiki.searchbar.return_text();
                    let wiki = wikipedia::Wikipedia::<wikipedia::http::default::Client>::default();
                    let page = wiki.page_from_title(text);
                    if let Ok(content) = page.get_content() {
                        self.create_source(conn, content);
                        self.menu = Menu::Main;
                    }
                }
                key => wiki.searchbar.keyhandler(key),
            }
            return;
        }

        match (&self.selection, key) {
            (Extracts, Enter) => self.new_focus(conn),
            (Extracts, Char('k')) | (Extracts, Up) => self.extracts.previous(),
            (Extracts, Char('j')) | (Extracts, Down) => self.extracts.next(),
            (Topics, key) => {
                self.topics.keyhandler(key, conn);
                self.reload_inc_list(conn);
            }
            (List, Enter) => self.new_focus(conn),
            (List, Char('k')) | (List, Up) => self.inclist.previous(),
            (List, Char('j')) | (List, Down) => self.inclist.next(),
            (Incread, key) => {
                if let Some(focused) = &mut self.focused {
                    let incid = focused.id;
                    focused.keyhandler(conn, key.clone());
                    if let MyKey::Alt('x') = &key {
                        self.reload_extracts(conn, incid)
                    }
                }
            }
            (_, Alt('w')) => {
                let wiki = WikiSelect::new();
                self.menu = Menu::WikiSelect(wiki);
            }
            (_, _) => {}
        }
    }

    fn nav_inc(&mut self, conn: &Arc<Mutex<Connection>>, dir: &Direction) {
        use crate::Direction::*;
        use Selection::*;

        let focused = self.focused.is_some();
        // println!("@");

        match (&self.selection, dir) {
            (Incread, Right) => {
                self.update_text(conn);
                self.selection = Topics;
                self.reload_inc_list(conn);
            }
            (Topics, Down) => self.selection = List,
            (List, Up) => self.selection = Topics,
            (List, Down) => self.selection = Extracts,
            (Extracts, Up) => self.selection = List,
            (_, Left) if focused => self.selection = Incread,
            _ => {}
        }
    }
}

impl<T> StraitList<T> for StatefulList<IncListItem> {
    fn state(&self) -> ListState {
        self.state.clone()
    }

    fn generate_list_items(&self, selected: bool, title: String) -> List {
        use tui::style::Modifier;
        use tui::text::Span;
        let bordercolor = if selected { Color::Red } else { Color::White };
        let style = Style::default().fg(bordercolor);

        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|inc| {
                let lines;
                if inc.text.len() < 3 {
                    lines = vec![Spans::from(Span::styled(
                        "Empty Source",
                        Style::default().add_modifier(Modifier::DIM | Modifier::ITALIC),
                    ))];
                } else {
                    lines = vec![Spans::from(inc.text.clone())];
                }
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
