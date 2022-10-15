use crate::utils::statelist::StatefulList;
use crate::utils::{
    aliases::*,
    card::Card,
    sql::{fetch::load_card_matches, insert::update_both},
};
use crate::widgets::textinput::Field;
use rusqlite::Connection;
use tui::style::Style;
use tui::{
    layout::{Constraint, Direction::Vertical, Layout, Rect},
    Frame,
};

use super::message_box::draw_message;
use crate::utils::misc::PopUpStatus;
use crate::{MyKey, MyType};
use std::sync::{Arc, Mutex};

pub struct FindCardWidget {
    pub prompt: String,
    pub searchterm: Field,
    pub list: StatefulList<CardMatch>,
    pub status: PopUpStatus,
    pub purpose: CardPurpose,
}

#[derive(Clone, PartialEq)]
pub struct CardMatch {
    pub question: String,
    pub id: CardID,
}

use std::fmt;
impl fmt::Display for CardMatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.question)
    }
}

pub enum CardPurpose {
    NewDependency(CardID),
    NewDependent(CardID),
    NewCloze(TopicID),
}

impl FindCardWidget {
    pub fn new(conn: &Arc<Mutex<Connection>>, prompt: String, purpose: CardPurpose) -> Self {
        let mut list = StatefulList::<CardMatch>::new();
        let searchterm = Field::new();
        list.reset_filter(conn, searchterm.return_text());

        let status = PopUpStatus::OnGoing;

        FindCardWidget {
            prompt,
            searchterm,
            list,
            status,
            purpose,
        }
    }

    pub fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey) -> PopUpStatus {
        match key {
            MyKey::Enter => self.complete(conn),
            MyKey::Esc => self.status = PopUpStatus::Finished,
            MyKey::Down => self.list.next(),
            MyKey::Up => self.list.previous(),
            key => {
                self.searchterm.keyhandler(key);
                self.list.reset_filter(conn, self.searchterm.return_text());
            }
        }
        self.status.clone()
    }

    fn complete(&mut self, conn: &Arc<Mutex<Connection>>) {
        if self.list.state.selected().is_none() {
            return;
        }

        let idx = self.list.state.selected().unwrap();
        let chosen_id = self.list.items[idx].id;

        match self.purpose {
            CardPurpose::NewDependent(source_id) => {
                update_both(conn, chosen_id, source_id).unwrap();
                Card::check_resolved(chosen_id, conn);
            }
            CardPurpose::NewDependency(source_id) => {
                update_both(conn, source_id, chosen_id).unwrap();
                Card::check_resolved(source_id, conn);
            }
            CardPurpose::NewCloze(_topic_id) => {
                todo!();
            }
        }
        self.status = PopUpStatus::Finished;
    }

    pub fn render(&mut self, f: &mut Frame<MyType>, area: Rect) {
        let chunks = Layout::default()
            .direction(Vertical)
            .constraints(
                [
                    Constraint::Ratio(1, 10),
                    Constraint::Ratio(1, 10),
                    Constraint::Ratio(8, 10),
                ]
                .as_ref(),
            )
            .split(area);

        let (prompt, searchbar, matchlist) = (chunks[0], chunks[1], chunks[2]);

        draw_message(f, prompt, &self.prompt);
        self.searchterm.render(f, searchbar, false);
        self.list.render(f, matchlist, false, "", Style::default());
    }
}

impl StatefulList<CardMatch> {
    pub fn reset_filter(&mut self, conn: &Arc<Mutex<Connection>>, mut searchterm: String) {
        let mut matching_cards = Vec::<CardMatch>::new();
        searchterm.pop();
        let all_cards = load_card_matches(conn, &searchterm).unwrap();
        for card in all_cards {
            matching_cards.push(CardMatch {
                question: card.question,
                id: card.id,
            });
        }
        self.items = matching_cards;
    }

    pub fn choose_card(&self) -> u32 {
        let index = self.state.selected().unwrap();
        self.items[index].id
    }
}
