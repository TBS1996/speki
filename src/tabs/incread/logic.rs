use crate::app::AppData;
use crate::app::PopUp;
use crate::app::Tab;
use crate::app::Widget;
use crate::utils::misc::split_leftright_by_percent;
use crate::utils::misc::split_updown_by_percent;
use crate::utils::misc::View;
use crate::utils::sql::update::update_inc_text;
use crate::MyKey;
use crate::MyType;

use crate::utils::aliases::*;
use crate::utils::statelist::StatefulList;
use crate::widgets::message_box::draw_message;
use crate::widgets::topics::TopicList;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::Clear;
use tui::Frame;

use crate::utils::incread::IncListItem;
use crate::utils::incread::IncRead;
use crate::widgets::textinput::Field;
use std::sync::{Arc, Mutex};

pub struct WikiSelect {
    pub searchbar: Field,
    prompt: String,
    topic: TopicID,
    should_quit: bool,
}

impl WikiSelect {
    fn new(id: TopicID) -> Self {
        WikiSelect {
            searchbar: Field::default(),
            prompt: "Search for a wikipedia page".to_string(),
            topic: id,
            should_quit: false,
        }
    }
}

impl PopUp for WikiSelect {
    fn should_quit(&self) -> bool {
        self.should_quit
    }
}

impl Widget for WikiSelect {
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        match key {
            MyKey::Esc => self.should_quit = true,
            MyKey::Enter => {
                let text = self.searchbar.return_text();
                let wiki = wikipedia::Wikipedia::<wikipedia::http::default::Client>::default();
                let page = wiki.page_from_title(text);
                if let Ok(content) = page.get_content() {
                    new_incread(&appdata.conn, 0, self.topic, content, true).unwrap();
                    self.should_quit = true;
                } else {
                    self.prompt = "Invalid search result".to_string();
                }
            }
            key => self.searchbar.keyhandler(key),
        }
    }

    fn render(&mut self, f: &mut Frame<MyType>, _appdata: &AppData, mut area: Rect) {
        if area.height > 10 && area.width > 10 {
            area = crate::utils::misc::centered_rect(80, 70, area);
            f.render_widget(Clear, area); //this clears out the background
            area.x += 2;
            area.y += 2;
            area.height -= 4;
            area.width -= 4;
        }
        let chunks = split_updown_by_percent([50, 50], area);
        let (mut msg, mut search) = (chunks[0], chunks[1]);
        msg.y = search.y - 5;
        msg.height = 5;
        search.height = 3;
        draw_message(f, msg, &self.prompt);
        self.searchbar.render(f, search, false);
    }
}

pub struct MainInc {
    pub inclist: StatefulList<IncListItem>,
    pub focused: Option<IncRead>,
    pub extracts: StatefulList<IncListItem>,
    pub topics: TopicList,
    pub popup: Option<Box<dyn PopUp>>,
    view: View,
}

use crate::utils::sql::fetch::load_extracts;
use crate::utils::sql::fetch::{get_incread, load_inc_items};
use crate::utils::sql::insert::new_incread;
use rusqlite::Connection;

impl MainInc {
    pub fn new(conn: &Arc<Mutex<Connection>>) -> Self {
        let items = load_inc_items(conn, 1).unwrap();
        let inclist = StatefulList::with_items(items);
        let extracts = StatefulList::<IncListItem>::new();
        let topics = TopicList::new(conn);
        let focused: Option<IncRead> = None;
        let popup = None;
        let view = View::default();

        MainInc {
            inclist,
            focused,
            extracts,
            topics,
            popup,
            view,
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
        self.inclist = StatefulList::with_items(items);
    }

    pub fn reload_extracts(&mut self, conn: &Arc<Mutex<Connection>>, id: IncID) {
        self.extracts.items = load_extracts(conn, id).unwrap();
    }
    pub fn create_source(&mut self, conn: &Arc<Mutex<Connection>>, text: String) {
        let topic: TopicID = self.topics.get_selected_id().unwrap();
        new_incread(conn, 0, topic, text, true).unwrap();
        self.reload_inc_list(conn);
    }

    fn set_selection(&mut self, area: Rect) {
        let chunks = split_leftright_by_percent([75, 15], area);
        let (left, right) = (chunks[0], chunks[1]);
        let right = split_updown_by_percent([33, 33, 33], right);
        let (topright, middleright, bottomright) = (right[0], right[1], right[2]);

        if let Some(inc) = &mut self.focused {
            inc.source.set_dimensions(left);
        }

        self.view.areas.insert("focused", left);
        self.view.areas.insert("topics", topright);
        self.view.areas.insert("inclist", middleright);
        self.view.areas.insert("extracts", bottomright);
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

        if let Some(popup) = &mut self.popup {
            popup.keyhandler(appdata, key);
            if popup.should_quit() {
                self.popup = None;
                self.reload_inc_list(&appdata.conn);
            }
            return;
        }

        if let MyKey::Nav(dir) = key {
            self.view.navigate(dir);
            return;
        } else if let MyKey::Alt('a') = &key {
            self.create_source(&appdata.conn, "".to_string());
            return;
        }

        match key {
            KeyPress(pos) if self.popup.is_none() => self.view.cursor = pos,
            Enter if self.view.name_selected("extracts") => self.focus_extracts(&appdata.conn),
            Enter if self.view.name_selected("inclist") => self.focus_list(&appdata.conn),
            Alt('w') => {
                let topic: TopicID = self.topics.get_selected_id().unwrap();
                let wiki = WikiSelect::new(topic);
                self.popup = Some(Box::new(wiki));
            }
            key if self.view.name_selected("extracts") => self.extracts.keyhandler(key),
            key if self.view.name_selected("inclist") => self.inclist.keyhandler(key),
            key if self.view.name_selected("topics") => {
                self.topics.keyhandler(key, &appdata.conn);
                self.reload_inc_list(&appdata.conn);
            }
            key if self.view.name_selected("focused") => {
                if let Some(focused) = &mut self.focused {
                    let incid = focused.id;
                    focused.keyhandler(&appdata.conn, key.clone());
                    if let MyKey::Alt('x') = &key {
                        self.reload_extracts(&appdata.conn, incid)
                    }
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, area: Rect) {
        self.set_selection(area);

        match &mut self.focused {
            Some(incread) => incread.source.render(
                f,
                self.view.get_area("focused"),
                self.view.name_selected("focused"),
            ),
            None => draw_message(f, self.view.get_area("focused"), "No text selected"),
        };

        self.topics.render(
            f,
            self.view.get_area("topics"),
            self.view.name_selected("topics"),
            "Topics",
            Style::default(),
        );

        self.inclist.render(
            f,
            self.view.get_area("inclist"),
            self.view.name_selected("inclist"),
            "Sources",
            Style::default(),
        );

        self.extracts.render(
            f,
            self.view.get_area("extracts"),
            self.view.name_selected("extracts"),
            "Extracts",
            Style::default(),
        );

        if let Some(popup) = &mut self.popup {
            popup.render(f, appdata, area);
        }
    }
}
