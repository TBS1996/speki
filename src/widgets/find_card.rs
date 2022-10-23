use crate::app::{AppData, PopUp, Widget};
use crate::utils::sql::fetch::CardQuery;
use crate::utils::statelist::{KeyHandler, StatefulList};
use crate::utils::{aliases::*, card::Card, sql::insert::update_both};
use crate::widgets::textinput::Field;
use rusqlite::Connection;
use tui::style::Style;
use tui::{
    layout::{Constraint, Direction::Vertical, Layout, Rect},
    Frame,
};

use super::message_box::draw_message;
use crate::{MyKey, MyType};
use std::sync::{Arc, Mutex};

pub struct FindCardWidget {
    pub prompt: String,
    pub searchterm: Field,
    pub list: StatefulList<CardMatch>,
    pub purpose: CardPurpose,
    pub should_quit: bool,
}

#[derive(Clone, PartialEq)]
pub struct CardMatch {
    pub question: String,
    pub id: CardID,
}

impl KeyHandler for CardMatch {}

use std::fmt;
impl fmt::Display for CardMatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.question)
    }
}

pub enum CardPurpose {
    NewDependency(Vec<CardID>),
    NewDependent(Vec<CardID>),
    NewCloze(TopicID),
}

impl FindCardWidget {
    pub fn new(conn: &Arc<Mutex<Connection>>, purpose: CardPurpose) -> Self {
        let mut list = StatefulList::<CardMatch>::new();
        let searchterm = Field::new();
        list.reset_filter(conn, searchterm.return_text());
        let should_quit = false;
        let prompt = match purpose {
            CardPurpose::NewDependent(_) => "Add new dependent".to_string(),
            CardPurpose::NewDependency(_) => "Add new dependency".to_string(),
            _ => panic!(),
        };

        FindCardWidget {
            prompt,
            searchterm,
            list,
            purpose,
            should_quit,
        }
    }

    fn complete(&mut self, conn: &Arc<Mutex<Connection>>) {
        if self.list.state.selected().is_none() {
            return;
        }

        let idx = self.list.state.selected().unwrap();
        let chosen_id = self.list.items[idx].id;

        match &self.purpose {
            CardPurpose::NewDependent(ids) => {
                for id in ids {
                    update_both(conn, chosen_id, *id).unwrap();
                    Card::check_resolved(chosen_id, conn);
                }
            }
            CardPurpose::NewDependency(ids) => {
                for id in ids {
                    update_both(conn, *id, chosen_id).unwrap();
                    Card::check_resolved(*id, conn);
                }
            }
            CardPurpose::NewCloze(_topic_id) => {
                todo!();
            }
        }
        self.should_quit = true;
    }
}

impl PopUp for FindCardWidget {
    fn should_quit(&self) -> bool {
        self.should_quit
    }
}

impl Widget for FindCardWidget {
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        match key {
            MyKey::Enter => self.complete(&appdata.conn),
            MyKey::Esc => self.should_quit = true,
            MyKey::Down => self.list.next(),
            MyKey::Up => self.list.previous(),
            key => {
                self.searchterm.keyhandler(key);
                self.list
                    .reset_filter(&appdata.conn, self.searchterm.return_text());
            }
        }
    }

    fn render(&mut self, f: &mut Frame<MyType>, _appdata: &AppData, area: Rect) {
        let chunks = Layout::default()
            .direction(Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(area);

        let (prompt, searchbar, matchlist) = (chunks[0], chunks[1], chunks[2]);
        draw_message(f, prompt, &self.prompt);
        self.searchterm.render(f, searchbar, false);
        self.list.render(f, matchlist, true, "", Style::default());
    }
}

impl StatefulList<CardMatch> {
    pub fn reset_filter(&mut self, conn: &Arc<Mutex<Connection>>, mut searchterm: String) {
        let mut matching_cards = Vec::<CardMatch>::new();
        searchterm.pop();
        let all_cards = CardQuery::default()
            .contains(searchterm)
            .limit(1000) //arbitrary
            .fetch_generic(conn, |row| CardMatch {
                question: row.get(1).unwrap(),
                id: row.get(0).unwrap(),
            });
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
