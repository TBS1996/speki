use super::aliases::*;
use super::card::{Card, CardTypeData, FinishedInfo};
use super::sql::fetch::{get_incread, load_extracts, CardQuery};
use super::sql::insert::new_incread;
use super::statelist::KeyHandler;
use crate::app::{AppData, Widget};
use crate::utils::card::CardItem;
use crate::utils::sql::update::update_inc_text;
use crate::utils::statelist::StatefulList;
use crate::widgets::textinput::Field;
use crate::MyKey;
use rusqlite::Connection;
use std::fmt::{self, Display};
use std::sync::{Arc, Mutex};

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

pub struct IncRead {
    pub id: IncID,
    pub parent: IncID,
    pub topic: TopicID,
    pub source: Field,
    pub extracts: StatefulList<IncListItem>,
    pub clozes: StatefulList<CardItem>,
    pub isactive: bool,
}

impl IncRead {
    pub fn new(conn: &Arc<Mutex<Connection>>, id: IncID) -> Self {
        get_incread(conn, id).unwrap()
    }

    pub fn extract(&mut self, conn: &Arc<Mutex<Connection>>) {
        if let Some(extract) = self.source.return_selection() {
            new_incread(conn, self.id, self.topic, extract, true).unwrap();
            self.extracts = StatefulList::with_items(
                "Extracts".to_string(),
                load_extracts(conn, self.id).unwrap(),
            );
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

            let cloze_cards = CardQuery::default().source(self.id).fetch_carditems(conn);
            self.clozes = StatefulList::with_items("Clozes".to_string(), cloze_cards);
        }
    }
    pub fn update_text(&self, conn: &Arc<Mutex<Connection>>) {
        let text = self.source.return_text();
        update_inc_text(&conn, text, self.id, &self.source.cursor).unwrap();
    }
}

impl Widget for IncRead {
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        match key {
            MyKey::Alt('x') => {
                self.extract(&appdata.conn);
                self.source.set_normal_mode();
            }
            MyKey::Alt('z') => {
                self.cloze(&appdata.conn);
                self.source.set_normal_mode();
            }
            MyKey::Esc => {
                self.update_text(&appdata.conn);
            }
            key => self.source.keyhandler(appdata, key),
        }
    }
    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        cursor: &(u16, u16),
    ) {
        self.source.render(f, appdata, cursor);
        self.extracts.render(f, appdata, cursor);
        self.clozes.render(f, appdata, cursor);
    }

    fn get_area(&self) -> tui::layout::Rect {
        self.source.get_area()
    }
    fn set_area(&mut self, area: tui::layout::Rect) {
        self.source.set_area(area);
    }
}
