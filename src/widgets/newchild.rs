use crate::app::{AppData, PopUp, Tab, Widget};
use crate::utils::card::{CardTypeData, CardView, FinishedInfo, UnfinishedInfo};
use crate::utils::misc::View;
use crate::utils::sql::fetch::get_topic_of_card;
use crate::utils::{aliases::*, card::Card};
use rusqlite::Connection;
use tui::{
    layout::{Constraint, Direction::Vertical, Layout, Rect},
    Frame,
};

use crate::utils::sql::fetch::get_topic_of_inc;
use crate::MyKey;
use crate::{MyType, NavDir};

use std::sync::{Arc, Mutex};

use super::button::Button;

pub enum Purpose {
    Source(TopicID),
    Dependency(Vec<CardID>),
    Dependent(Vec<CardID>),
}

pub struct AddChildWidget {
    pub prompt: Button,
    pub cardview: CardView,
    pub purpose: Purpose,
    pub should_quit: bool,
    view: View,
}

impl AddChildWidget {
    pub fn new(conn: &Arc<Mutex<Connection>>, purpose: Purpose) -> Self {
        let prompt = Self::add_prompt(conn, &purpose);
        let cardview = CardView::default();
        let should_quit = false;
        let view = View::default();

        AddChildWidget {
            prompt,
            cardview,
            purpose,
            should_quit,
            view,
        }
    }

    fn add_prompt(_conn: &Arc<Mutex<Connection>>, purpose: &Purpose) -> Button {
        let text = match purpose {
            Purpose::Source(_) => Button::new("Add new sourced card".to_string()),
            Purpose::Dependency(_) => Button::new("Add new dependent".to_string()),
            Purpose::Dependent(_) => Button::new("Add new dependency".to_string()),
        };
        text
    }

    fn submit_card(&mut self, conn: &Arc<Mutex<Connection>>, isfinished: bool) {
        let topic = match &self.purpose {
            Purpose::Source(id) => get_topic_of_inc(conn, *id).unwrap(),
            Purpose::Dependent(id) => get_topic_of_card(conn, id[0]),
            Purpose::Dependency(id) => get_topic_of_card(conn, id[0]),
        };

        let question = self.cardview.question.return_text();
        let answer = self.cardview.answer.return_text();
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

impl Tab for AddChildWidget {
    fn get_title(&self) -> String {
        "Add card".to_string()
    }
    fn set_selection(&mut self, area: Rect) {
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
        self.view.areas.clear();
        self.view.areas.push(prompt);
        self.view.areas.push(question);
        self.view.areas.push(answer);

        if self.cardview.question.get_area().width == 0 {
            self.view.move_to_area(question);
        }

        self.prompt.set_area(prompt);
        self.cardview.question.set_area(question);
        self.cardview.answer.set_area(answer);
    }
    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, area: Rect) {
        self.set_selection(area);
        let cursor = self.get_cursor().clone();
        self.prompt.render(f, appdata, &cursor);
        self.cardview.question.render(f, appdata, &cursor);
        self.cardview.answer.render(f, appdata, &cursor);
    }
    fn get_cursor(&self) -> (u16, u16) {
        self.view.cursor
    }

    fn navigate(&mut self, dir: NavDir) {
        self.view.navigate(dir);
    }

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        use MyKey::*;
        let cursor = &self.get_cursor();
        match key {
            Esc => self.should_quit = true,
            KeyPress(pos) => self.view.cursor = pos,
            Alt('f') => self.submit_card(&appdata.conn, true),
            Alt('u') => self.submit_card(&appdata.conn, false),
            key => self.cardview.keyhandler(appdata, cursor, key),
        }
    }
}
