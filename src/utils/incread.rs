use super::aliases::*;
use super::card::Card;
use super::sql::fetch::{get_incread, load_extracts, CardQuery};
use super::sql::insert::new_incread;
use crate::utils::sql::update::update_inc_text;
use crate::utils::statelist::StatefulList;
use crate::widgets::cardlist::CardItem;
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

impl Display for IncListItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
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
        get_incread(&conn, id).unwrap()
    }

    pub fn extract(&mut self, conn: &Arc<Mutex<Connection>>) {
        if let Some(extract) = self.source.return_selection() {
            new_incread(&conn, self.id, self.topic, extract, true).unwrap();
            self.extracts = StatefulList::with_items(load_extracts(&conn, self.id).unwrap());
        }
    }
    pub fn cloze(&mut self, conn: &Arc<Mutex<Connection>>) {
        if let Some(cloze) = self.source.return_selection() {
            let mut question = self.source.return_text();
            question = question.replace(&cloze, "[...]");
            let answer = cloze;

            Card::new()
                .question(question)
                .answer(answer)
                .topic(self.topic)
                .source(self.id)
                .cardtype(super::card::CardType::Finished)
                .save_card(conn);

            let cloze_cards = CardQuery::default().source(self.id).fetch_carditems(conn);
            self.clozes = StatefulList::with_items(cloze_cards);
        }
    }
    pub fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey) {
        match key {
            MyKey::Alt('x') => {
                self.extract(conn);
                self.source.set_normal_mode();
            }
            MyKey::Alt('z') => {
                self.cloze(conn);
                self.source.set_normal_mode();
            }
            MyKey::Esc => {
                self.update_text(conn);
            }
            key => self.source.keyhandler(key),
        }
    }
    pub fn update_text(&self, conn: &Arc<Mutex<Connection>>) {
        let text = self.source.return_text();
        update_inc_text(&conn, text, self.id, &self.source.cursor).unwrap();
    }
}
