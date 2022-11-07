use crate::app::AppData;
use crate::app::Tab;
use crate::app::TabData;
use crate::app::Widget;
use crate::popups::edit_text::TextEditor;
use crate::popups::wikiselect::WikiSelect;
use crate::utils::misc::split_leftright_by_percent;
use crate::MyKey;
use crate::MyType;

use crate::utils::aliases::*;
use crate::utils::sql::fetch::load_inc_items;
use crate::utils::statelist::StatefulList;
use crate::widgets::topics::TopicList;
use tui::layout::Rect;
use tui::Frame;

use crate::utils::incread::IncListItem;
use std::sync::{Arc, Mutex};

pub struct MainInc {
    pub inclist: StatefulList<IncListItem>,
    pub topics: TopicList,
    tabdata: TabData,
}

use rusqlite::Connection;

impl MainInc {
    pub fn new(conn: &Arc<Mutex<Connection>>) -> Self {
        let items = load_inc_items(conn, 1).unwrap();
        let inclist = StatefulList::with_items("Sources".to_string(), items);
        let topics = TopicList::new(conn);

        MainInc {
            inclist,
            topics,
            tabdata: TabData::default(),
        }
    }
    /*
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

    */

    pub fn reload_inc_list(&mut self, conn: &Arc<Mutex<Connection>>) {
        let items = load_inc_items(conn, self.topics.get_selected_id().unwrap()).unwrap();
        self.inclist = StatefulList::with_items("Sources".to_string(), items);
    }
}

impl Tab for MainInc {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn set_selection(&mut self, area: Rect) {
        let chunks = split_leftright_by_percent([25, 75], area);

        self.tabdata.view.areas.push(chunks[0]);
        self.tabdata.view.areas.push(chunks[1]);

        self.topics.set_area(chunks[0]);
        self.inclist.set_area(chunks[1]);
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

        match key {
            //MyKey::Alt('a') => self.create_source(&appdata.conn, "".to_string()),
            Alt('w') => {
                let topic: TopicID = self.topics.get_selected_id().unwrap();
                let wiki = WikiSelect::new(topic);
                self.set_popup(Box::new(wiki));
            }
            Enter if self.inclist.is_selected(cursor) => {
                if let Some(idx) = self.inclist.state.selected() {
                    let id = self.inclist.items[idx].id;
                    let txt = TextEditor::new(appdata, id);
                    self.set_popup(Box::new(txt));
                }
            }
            key if self.inclist.is_selected(cursor) => self.inclist.keyhandler(appdata, key),
            key if self.topics.is_selected(cursor) => {
                self.topics.keyhandler(appdata, key);
                self.reload_inc_list(&appdata.conn);
            }

            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &(u16, u16)) {
        self.topics.render(f, appdata, cursor);
        self.inclist.render(f, appdata, cursor);
    }
}
