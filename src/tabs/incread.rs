use crate::app::AppData;
use crate::app::Tab;
use crate::app::TabData;
use crate::app::Widget;
use crate::popups::wikiselect::WikiSelect;
use crate::utils::misc::split_leftright_by_percent;
use crate::utils::misc::split_updown_by_percent;
use crate::utils::sql::update::update_inc_text;
use crate::MyKey;
use crate::MyType;

use crate::utils::aliases::*;
use crate::utils::statelist::StatefulList;
use crate::widgets::topics::TopicList;
use tui::layout::Rect;
use tui::Frame;

use crate::utils::incread::IncListItem;
use crate::utils::incread::IncRead;
use std::sync::{Arc, Mutex};

pub struct MainInc {
    pub inclist: StatefulList<IncListItem>,
    pub focused: Option<IncRead>,
    pub extracts: StatefulList<IncListItem>,
    pub topics: TopicList,
    tabdata: TabData,
}

use crate::utils::sql::fetch::load_extracts;
use crate::utils::sql::fetch::{get_incread, load_inc_items};
use crate::utils::sql::insert::new_incread;
use rusqlite::Connection;

impl MainInc {
    pub fn new(conn: &Arc<Mutex<Connection>>) -> Self {
        let items = load_inc_items(conn, 1).unwrap();
        let inclist = StatefulList::with_items("Sources".to_string(), items);
        let extracts = StatefulList::<IncListItem>::new("Extracts".to_string());
        let topics = TopicList::new(conn);
        let focused: Option<IncRead> = None;

        MainInc {
            inclist,
            focused,
            extracts,
            topics,
            tabdata: TabData::default(),
        }
    }

    pub fn update_text(&self, conn: &Arc<Mutex<Connection>>) {
        if let Some(inc) = &self.focused {
            let id = inc.id;
            let text = inc.source.return_text();
            update_inc_text(conn, text, id, &inc.source.cursor).unwrap();
        }
    }

    fn focus_list(&mut self, conn: &Arc<Mutex<Connection>>) {
        if let Some(idx) = self.inclist.state.selected() {
            let id: IncID = self.inclist.items[idx].id;
            let incread = get_incread(conn, id).unwrap();
            self.focused = Some(incread);
            self.extracts.items = load_extracts(conn, id).unwrap();
        }
    }

    fn focus_extracts(&mut self, conn: &Arc<Mutex<Connection>>) {
        if let Some(idx) = self.extracts.state.selected() {
            let id: IncID = self.extracts.items[idx].id;
            let incread = get_incread(conn, id).unwrap();
            self.focused = Some(incread);
            self.extracts.items = load_extracts(conn, id).unwrap();
        }
    }

    pub fn reload_inc_list(&mut self, conn: &Arc<Mutex<Connection>>) {
        let items = load_inc_items(conn, self.topics.get_selected_id().unwrap()).unwrap();
        self.inclist = StatefulList::with_items("Sources".to_string(), items);
    }

    pub fn reload_extracts(&mut self, conn: &Arc<Mutex<Connection>>, id: IncID) {
        self.extracts.items = load_extracts(conn, id).unwrap();
    }
    pub fn create_source(&mut self, conn: &Arc<Mutex<Connection>>, text: String) {
        let topic: TopicID = self.topics.get_selected_id().unwrap();
        new_incread(conn, 0, topic, text, true).unwrap();
        self.reload_inc_list(conn);
    }
}

impl Tab for MainInc {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn set_selection(&mut self, area: Rect) {
        let chunks = split_leftright_by_percent([75, 15], area);
        let (left, right) = (chunks[0], chunks[1]);
        let right = split_updown_by_percent([33, 33, 33], right);
        let (topright, middleright, bottomright) = (right[0], right[1], right[2]);

        if let Some(inc) = &mut self.focused {
            inc.source.set_dimensions(left);
        }

        self.tabdata.view.areas.push(left);
        self.tabdata.view.areas.push(topright);
        self.tabdata.view.areas.push(middleright);
        self.tabdata.view.areas.push(bottomright);

        if let Some(inc) = &mut self.focused {
            inc.source.set_area(left);
        }
        self.topics.set_area(topright);
        self.extracts.set_area(bottomright);
        self.inclist.set_area(middleright);
    }
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

    fn exit_popup(&mut self, appdata: &AppData) {
        self.tabdata.popup = None;
        self.reload_inc_list(&appdata.conn);
    }

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey, cursor: &(u16, u16)) {
        use crate::MyKey::*;

        let incfocus = {
            if let Some(inc) = &mut self.focused {
                inc.source.is_selected(cursor)
            } else {
                false
            }
        };

        match key {
            MyKey::Alt('a') => self.create_source(&appdata.conn, "".to_string()),
            Enter if self.extracts.is_selected(cursor) => self.focus_extracts(&appdata.conn),
            Enter if self.inclist.is_selected(cursor) => self.focus_list(&appdata.conn),
            Alt('w') => {
                let topic: TopicID = self.topics.get_selected_id().unwrap();
                let wiki = WikiSelect::new(topic);
                self.set_popup(Box::new(wiki));
            }
            key if self.extracts.is_selected(cursor) => self.extracts.keyhandler(appdata, key),
            key if self.inclist.is_selected(cursor) => self.inclist.keyhandler(appdata, key),
            key if self.topics.is_selected(cursor) => {
                self.topics.keyhandler(appdata, key);
                self.reload_inc_list(&appdata.conn);
            }
            key if incfocus => {
                if let Some(focused) = &mut self.focused {
                    let incid = focused.id;
                    focused.keyhandler(appdata, key.clone());
                    if let MyKey::Alt('x') = &key {
                        self.reload_extracts(&appdata.conn, incid)
                    }
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &(u16, u16)) {
        if let Some(inc) = &mut self.focused {
            inc.source.render(f, appdata, cursor);
        }
        self.topics.render(f, appdata, cursor);
        self.inclist.render(f, appdata, cursor);
        self.extracts.render(f, appdata, cursor);
    }
}
