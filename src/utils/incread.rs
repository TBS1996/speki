use super::aliases::*;
use super::card::{Card, CardTypeData, FinishedInfo};
use super::misc::days_until_unix;
use super::sql::fetch::{get_incread, load_extracts, CardQuery};
use super::sql::insert::new_incread;
use super::sql::update::update_inc_active;
use super::statelist::KeyHandler;
use crate::app::{AppData, TabData, Widget};
use crate::popups::edit_card::Editor;
use crate::popups::edit_text::TextEditor;
use crate::popups::newchild::{AddChildWidget, Purpose};
use crate::utils::card::CardItem;
use crate::utils::sql::update::update_inc_text;
use crate::utils::statelist::StatefulList;
use crate::widgets::button::Button;
use crate::widgets::infobox::InfoBox;
use crate::widgets::textinput::Field;
use crate::widgets::topics::TopicList;
use crate::{MyKey, MyType};
use rusqlite::Connection;
use std::fmt::{self, Display};
use std::sync::{Arc, Mutex};
use tui::style::{Modifier, Style};
use tui::Frame;

#[derive(Clone)]
pub struct IncListItem {
    pub text: String,
    pub id: IncID,
}

impl KeyHandler for IncListItem {}

impl Display for IncListItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = if self.text.len() > 2 {
            self.text.clone()
        } else {
            String::from("--Empty source--")
        };

        write!(f, "{}", text)
    }
}

pub enum IncStatus {
    Done,
    Active(IncActive),
}

impl IncStatus {
    pub fn new_from_row(row: &rusqlite::Row<'_>) -> IncStatus {
        match row.get(4).unwrap() {
            0 => IncStatus::Done,
            1 => IncStatus::Active(IncActive {
                interval: row.get(6).unwrap(),
                timestamp: std::time::Duration::from_secs(row.get::<usize, u64>(5).unwrap()),
            }),
            _ => panic!(),
        }
    }
}

pub struct IncActive {
    interval: u32,
    timestamp: UnixTime,
}

pub struct IncRead {
    pub id: IncID,
    pub parent: IncID,
    pub topic: TopicID,
    pub status: IncStatus,
    pub source: Field,
    pub extracts: Vec<IncListItem>,
    pub clozes: Vec<CardItem>,
}

impl IncRead {
    pub fn new(conn: &Arc<Mutex<Connection>>, id: IncID) -> Self {
        get_incread(conn, id)
    }

    pub fn extract(&mut self, conn: &Arc<Mutex<Connection>>) {
        if let Some(extract) = self.source.return_selection() {
            new_incread(conn, self.id, self.topic, extract, true);
            self.extracts = load_extracts(conn, self.id);
        }
    }
    pub fn cloze(&mut self, conn: &Arc<Mutex<Connection>>) {
        if let Some(cloze) = self.source.return_selection() {
            let mut question = self.source.return_text();
            question = question.replace(&cloze, "[...]");
            let answer = cloze;

            Card::new(CardTypeData::Finished(FinishedInfo::default()))
                .question(question)
                .answer(answer)
                .topic(self.topic)
                .source(self.id)
                .save_card(conn);
            self.clozes = CardQuery::default().source(self.id).fetch_carditems(conn);
        }
    }
    pub fn update_text(&self, conn: &Arc<Mutex<Connection>>) {
        let text = self.source.return_text();
        update_inc_text(&conn, text, self.id, &self.source.cursor).unwrap();
    }
}

pub struct IncView<'a> {
    pub text: IncRead,
    pub topics: TopicList,
    pub extracts: StatefulList<IncListItem>,
    pub clozes: StatefulList<CardItem>,
    pub parent: Button<'a>,
    pub info: InfoBox<'a>,
}

impl<'a> IncView<'a> {
    pub fn new(appdata: &AppData, id: IncID) -> Self {
        let text = IncRead::new(&appdata.conn, id);
        let topics = TopicList::new(&appdata.conn);
        let extracts = StatefulList::with_items("Extracts".to_string(), text.extracts.clone());
        let clozes = StatefulList::with_items("Clozes".to_string(), text.clozes.clone());
        let mut parent = Button::new("Go to parent".to_string());
        let parent_id = text.parent;
        if parent_id == 0 {
            parent.inner.textstyle = Style::default().add_modifier(Modifier::DIM);
        }
        let info = InfoBox::new("hey there".to_string());
        Self {
            text,
            topics,
            extracts,
            clozes,
            parent,
            info,
        }
    }

    pub fn change_inc(&mut self, appdata: &AppData, id: IncID) {
        self.save_state(appdata);
        let inc = get_incread(&appdata.conn, id);
        self.text = inc;

        self.extracts =
            StatefulList::with_items("Extracts".to_string(), self.text.extracts.clone());
        self.clozes = StatefulList::with_items("Clozes".to_string(), self.text.clozes.clone());
    }

    pub fn prev_sibling(&mut self, appdata: &AppData) {
        let parent = self.text.parent;
        let incs = load_extracts(&appdata.conn, parent);
        for (idx, inc) in incs.iter().enumerate() {
            if inc.id == self.text.id {
                if idx != 0 {
                    self.change_inc(appdata, incs[idx - 1].id);
                }
                return;
            }
        }
    }

    pub fn next_sibling(&mut self, appdata: &AppData) {
        let parent = self.text.parent;
        let incs = load_extracts(&appdata.conn, parent);
        for (idx, inc) in incs.iter().enumerate() {
            if inc.id == self.text.id {
                if idx != incs.len() - 1 {
                    self.change_inc(appdata, incs[idx + 1].id);
                }
                return;
            }
        }
    }

    pub fn save_state(&mut self, appdata: &AppData) {
        self.text.update_text(&appdata.conn);
    }

    pub fn keyhandler(
        &mut self,
        appdata: &AppData,
        tabdata: &mut TabData,
        cursor: &Pos,
        key: MyKey,
    ) {
        match key {
            MyKey::Alt('a') => {
                let tab = AddChildWidget::new(appdata, Purpose::Source(self.text.id));
                tabdata.popup = Some(Box::new(tab));
            }
            MyKey::Alt('d') => {
                let id = self.text.id;
                update_inc_active(&appdata.conn, id, false).unwrap();
            }
            MyKey::Up if self.info.is_selected(cursor) => {
                let parent = self.text.parent;
                if parent != 0 {
                    let inc = TextEditor::new(appdata, parent);
                    tabdata.popup = Some(Box::new(inc));
                }
            }
            MyKey::Right if self.info.is_selected(cursor) => self.next_sibling(appdata),
            MyKey::Left if self.info.is_selected(cursor) => self.prev_sibling(appdata),
            MyKey::Char('e') | MyKey::Enter if self.extracts.is_selected(cursor) => {
                if let Some(idx) = self.extracts.state.selected() {
                    let id = self.extracts.items[idx].id;
                    tabdata.popup = Some(Box::new(TextEditor::new(appdata, id)));
                }
            }
            key if self.extracts.is_selected(cursor) => self.extracts.keyhandler(appdata, key),
            MyKey::Char('e') | MyKey::Enter if self.clozes.is_selected(cursor) => {
                if let Some(idx) = self.clozes.state.selected() {
                    let id = self.clozes.items[idx].id;
                    tabdata.popup = Some(Box::new(Editor::new(appdata, vec![id])));
                }
            }
            key if self.clozes.is_selected(cursor) => self.clozes.keyhandler(appdata, key),

            MyKey::Alt('x') if self.text.source.is_selected(cursor) => {
                self.text.extract(&appdata.conn);
                self.extracts =
                    StatefulList::with_items("Extracts".to_string(), self.text.extracts.clone());
                self.text.source.clear_selection();
            }
            MyKey::Alt('z') if self.text.source.is_selected(cursor) => {
                self.text.cloze(&appdata.conn);
                self.clozes =
                    StatefulList::with_items("Extracts".to_string(), self.text.clozes.clone());

                self.text.source.clear_selection();
            }
            key if self.text.source.is_selected(cursor) => {
                self.text.source.keyhandler(appdata, key)
            }
            key if self.topics.is_selected(cursor) => self.topics.keyhandler(appdata, key),

            _ => {}
        }
        self.update_info();
    }

    pub fn update_info(&mut self) {
        let text = match &self.text.status {
            IncStatus::Done => "This text is complete".to_string(),
            IncStatus::Active(val) => {
                format!(
                    "Text is active! Interval is {}, next review is in some days",
                    val.interval,
                )
            }
        };
        self.info.change_text(text);
    }

    pub fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &Pos) {
        self.text.source.render(f, appdata, cursor);
        self.extracts.render(f, appdata, cursor);
        self.clozes.render(f, appdata, cursor);
        self.topics.render(f, appdata, cursor);
        self.parent.render(f, appdata, cursor);
        self.info.render(f, appdata, cursor);
    }
}
