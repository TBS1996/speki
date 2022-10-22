use crate::app::AppData;
use crate::app::Tab;
use crate::app::Widget;
use crate::utils::sql::update::update_inc_text;
use crate::Direction;
use crate::MyKey;
use crate::MyType;

use crate::utils::aliases::*;
use crate::utils::statelist::StatefulList;
use crate::widgets::topics::TopicList;
use tui::layout::Rect;
use tui::Frame;

use crate::utils::incread::IncListItem;
use crate::utils::incread::IncRead;
use crate::widgets::textinput::Field;
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
        let inclist = StatefulList::with_items(items);
        let mut topics = TopicList::new(conn);
        topics.next();
        let focused: Option<IncRead> = None;
        let menu = Menu::Main;
        MainInc {
            inclist,
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
                let incread = get_incread(conn, id).unwrap();
                self.focused = Some(incread);
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
        self.inclist = StatefulList::with_items(items);
    }

    pub fn reload_extracts(&mut self, conn: &Arc<Mutex<Connection>>, id: IncID) {
        self.extracts.items = load_extracts(conn, id).unwrap();
    }

    fn nav_inc(&mut self, conn: &Arc<Mutex<Connection>>, dir: &Direction) {
        use crate::Direction::*;
        use Selection::*;

        let focused = self.focused.is_some();

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

impl Tab for MainInc {
    fn get_title(&self) -> String {
        "Incremental reading".to_string()
    }

    fn get_manual(&self) -> String {
        r#"

Sources are the top level texts with the topic that is currently selected.
Extracts are the extracts taken from the currently focused text.
You can paste text into the textwidget.

Add wikipedia page: Alt+w
add new source: Alt+a
insert mode -> normal mode: Ctrl+c
normal mode -> insert mode: i
normal mode -> visual mode: v
visual mode -> normal mode: Ctrl+c
make extract (visual mode): Alt+x 
make cloze (visual mode): Alt+z

        "#
        .to_string()
    }
}

impl Widget for MainInc {
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        use crate::MyKey::*;
        use Selection::*;
        if let MyKey::Nav(dir) = &key {
            self.nav_inc(&appdata.conn, dir);
            return;
        } else if let MyKey::Alt('a') = &key {
            self.create_source(&appdata.conn, "".to_string());
            return;
        } else if let Menu::WikiSelect(wiki) = &mut self.menu {
            match key {
                Esc => self.menu = Menu::Main,
                Enter => {
                    let text = wiki.searchbar.return_text();
                    let wiki = wikipedia::Wikipedia::<wikipedia::http::default::Client>::default();
                    let page = wiki.page_from_title(text);
                    if let Ok(content) = page.get_content() {
                        self.create_source(&appdata.conn, content);
                        self.menu = Menu::Main;
                    }
                }
                key => wiki.searchbar.keyhandler(key),
            }
            return;
        }

        match (&self.selection, key) {
            (Extracts, Enter) => self.new_focus(&appdata.conn),
            (Extracts, Char('k')) | (Extracts, Up) => self.extracts.previous(),
            (Extracts, Char('j')) | (Extracts, Down) => self.extracts.next(),
            (Topics, key) => {
                self.topics.keyhandler(key, &appdata.conn);
                self.reload_inc_list(&appdata.conn);
            }
            (List, Enter) => self.new_focus(&appdata.conn),
            (List, Char('k')) | (List, Up) => self.inclist.previous(),
            (List, Char('j')) | (List, Down) => self.inclist.next(),
            (Incread, key) => {
                if let Some(focused) = &mut self.focused {
                    let incid = focused.id;
                    focused.keyhandler(&appdata.conn, key.clone());
                    if let MyKey::Alt('x') = &key {
                        self.reload_extracts(&appdata.conn, incid)
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

    fn render(&mut self, f: &mut Frame<MyType>, _appdata: &AppData, area: Rect) {
        self.main_render(f, area);
    }
}
