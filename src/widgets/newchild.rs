use crate::app::{AppData, PopUp, Widget};
use crate::utils::card::{CardTypeData, FinishedInfo, UnfinishedInfo};
use crate::utils::sql::fetch::get_topic_of_card;
use crate::utils::{aliases::*, card::Card};
use crate::widgets::textinput::Field;
use rusqlite::Connection;
use tui::{
    layout::{Constraint, Direction::Vertical, Layout, Rect},
    Frame,
};

use crate::utils::sql::fetch::get_topic_of_inc;
use crate::MyKey;
use crate::{MyType, NavDir};

use std::sync::{Arc, Mutex};
pub enum Selection {
    Question,
    Answer,
}

pub enum Purpose {
    Source(TopicID),
    Dependency(Vec<CardID>),
    Dependent(Vec<CardID>),
}

pub struct AddChildWidget {
    pub prompt: Field,
    pub question: Field,
    pub answer: Field,
    pub selection: Selection,
    pub purpose: Purpose,
    pub should_quit: bool,
}

impl AddChildWidget {
    pub fn new(conn: &Arc<Mutex<Connection>>, purpose: Purpose) -> Self {
        let prompt = Self::add_prompt(conn, &purpose);
        let question = Field::default();
        let answer = Field::default();
        let selection = Selection::Question;
        let should_quit = false;

        AddChildWidget {
            prompt,
            question,
            answer,
            selection,
            purpose,
            should_quit,
        }
    }

    fn add_prompt(_conn: &Arc<Mutex<Connection>>, purpose: &Purpose) -> Field {
        let mut prompt = Field::default();
        match purpose {
            Purpose::Source(_) => {
                prompt.push("Add new sourced card".to_string());
            }
            Purpose::Dependency(_) => {
                prompt.push("Add new dependent".to_string());
            }
            Purpose::Dependent(_) => {
                prompt.push("Add new dependency".to_string());
            }
        }
        prompt
    }

    fn submit_card(&mut self, conn: &Arc<Mutex<Connection>>, isfinished: bool) {
        let topic = match &self.purpose {
            Purpose::Source(id) => get_topic_of_inc(conn, *id).unwrap(),
            Purpose::Dependent(id) => get_topic_of_card(conn, id[0]),
            Purpose::Dependency(id) => get_topic_of_card(conn, id[0]),
        };

        let question = self.question.return_text();
        let answer = self.answer.return_text();
        let source = if let Purpose::Source(id) = self.purpose {
            id
        } else {
            0
        };
        let status = if isfinished {
            CardTypeData::Finished(FinishedInfo::default())
        } else {
            CardTypeData::Unfinished(UnfinishedInfo::default())
        };

        let mut card = Card::new(status)
            .question(question)
            .answer(answer)
            .topic(topic)
            .source(source);

        match &self.purpose {
            Purpose::Dependent(cid) => {
                card = card.dependents(cid.clone());
            }
            Purpose::Dependency(cid) => {
                card = card.dependencies(cid.clone());
            }
            _ => {}
        }
        card.save_card(conn);
        self.should_quit = true;
    }
}

impl PopUp for AddChildWidget {
    fn should_quit(&self) -> bool {
        self.should_quit
    }
}

impl Widget for AddChildWidget {
    fn render(&mut self, f: &mut Frame<MyType>, _appdata: &AppData, area: Rect) {
        let chunks = Layout::default()
            .direction(Vertical)
            .constraints(
                [
                    Constraint::Ratio(2, 10),
                    Constraint::Ratio(4, 10),
                    Constraint::Ratio(4, 10),
                ]
                .as_ref(),
            )
            .split(area);

        let (prompt, question, answer) = (chunks[0], chunks[1], chunks[2]);

        let (mut qsel, mut asel) = (false, false);

        match self.selection {
            Selection::Question => qsel = true,
            Selection::Answer => asel = true,
        }

        self.prompt.render(f, prompt, false);
        self.question.render(f, question, qsel);
        self.answer.render(f, answer, asel);
    }

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        use MyKey::*;
        match key {
            Esc => self.should_quit = true,
            Alt('f') => self.submit_card(&appdata.conn, true),
            Alt('u') => self.submit_card(&appdata.conn, false),
            Nav(NavDir::Up) => self.selection = Selection::Question,
            Nav(NavDir::Down) => self.selection = Selection::Answer,
            key => match self.selection {
                Selection::Question => self.question.keyhandler(key),
                Selection::Answer => self.answer.keyhandler(key),
            },
        }
    }
}
