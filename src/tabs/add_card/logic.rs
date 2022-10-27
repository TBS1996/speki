use crate::app::AppData;
use crate::app::Tab;
use crate::app::Widget;
use crate::utils::card::Card;
use crate::utils::card::CardTypeData;
use crate::utils::card::FinishedInfo;
use crate::utils::card::UnfinishedInfo;
use crate::utils::misc::get_gpt3_response;
use crate::utils::misc::View;
use crate::utils::misc::{split_leftright_by_percent, split_updown_by_percent};
use crate::widgets::message_box::draw_message;
use crate::widgets::textinput::Field;
use crate::{MyKey, MyType};
use rusqlite::Connection;
use tui::layout::Rect;
use tui::style::Style;
use tui::Frame;

use crate::widgets::topics::TopicList;

//#[derive(Clone)]
pub struct NewCard {
    pub prompt: String,
    pub question: Field,
    pub answer: Field,
    pub topics: TopicList,
    view: View,
}

use std::sync::{Arc, Mutex};

impl NewCard {
    pub fn new(conn: &Arc<Mutex<Connection>>) -> NewCard {
        let topics = TopicList::new(conn);
        let view = View::default();

        NewCard {
            prompt: NewCard::make_prompt(conn),
            question: Field::new(),
            answer: Field::new(),
            topics,
            view,
        }
    }

    pub fn navigate(&mut self, dir: crate::NavDir) {
        use crate::NavDir::*;

        match dir {
            Right => self.view.move_right(),
            Left => self.view.move_left(),
            Up => self.view.move_up(),
            Down => self.view.move_down(),
        }
    }

    fn make_prompt(_conn: &Arc<Mutex<Connection>>) -> String {
        "Add new card".to_string()
    }

    pub fn submit_card(&mut self, conn: &Arc<Mutex<Connection>>, iscompleted: bool) {
        let question = self.question.return_text();
        let answer = self.answer.return_text();
        let topic = self.topics.get_selected_id().unwrap();
        let source = 0;

        let status = if iscompleted {
            CardTypeData::Finished(FinishedInfo::default())
        } else {
            CardTypeData::Unfinished(UnfinishedInfo::default())
        };

        //(conn, question, answer, topic, source, iscompleted);
        let card = Card::new(status)
            .question(question)
            .answer(answer)
            .topic(topic)
            .source(source);

        card.save_card(conn);
        *self = Self::new(conn);
    }

    pub fn uprow(&mut self) {}
    pub fn downrow(&mut self) {}
    pub fn home(&mut self) {}
    pub fn end(&mut self) {}

    fn set_selection(&mut self, area: Rect) {
        self.view.areas.clear();
        let chunks = split_leftright_by_percent([75, 15], area);
        let left = chunks[0];
        let right = chunks[1];
        self.view.areas.insert("topics", right);
        let chunks = split_updown_by_percent([10, 37, 37], left);
        self.view.areas.insert("prompt", chunks[0]);
        self.view.areas.insert("question", chunks[1]);
        self.view.areas.insert("answer", chunks[2]);
    }
}

impl Tab for NewCard {
    fn get_title(&self) -> String {
        "Add card".to_string()
    }
    fn get_manual(&self) -> String {
        r#"

Topic of card is as selected in the topic widget.

Upper textbox is question, lower is answer.

add card as finished: Alt+f
Add card as unfinished: Alt+u    

        "#
        .to_string()
    }
}

impl Widget for NewCard {
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        use MyKey::*;
        match key {
            Nav(dir) => self.navigate(dir),
            KeyPress(pos) => self.view.cursor = pos,
            Alt('f') => self.submit_card(&appdata.conn, true),
            Alt('u') => self.submit_card(&appdata.conn, false),
            Alt('g') => {
                if let Some(key) = &appdata.config.gptkey {
                    let answer = get_gpt3_response(key, &self.question.return_text());
                    self.answer.replace_text(answer);
                }
            }
            key if self.view.name_selected("question") => self.question.keyhandler(key),
            key if self.view.name_selected("answer") => self.answer.keyhandler(key),
            key if self.view.name_selected("topics") => self.topics.keyhandler(key, &appdata.conn),
            _ => {}
        }
    }
    fn render(&mut self, f: &mut Frame<MyType>, _appdata: &AppData, area: Rect) {
        self.set_selection(area);

        self.topics.render(
            f,
            *self.view.areas.get("topics").unwrap(),
            self.view.name_selected("topics"),
            "Topics",
            Style::default(),
        );
        draw_message(
            f,
            *self.view.areas.get("prompt").unwrap(),
            self.prompt.as_str(),
        );
        self.question.render(
            f,
            *self.view.areas.get("question").unwrap(),
            self.view.name_selected("question"),
        );
        self.answer.render(
            f,
            *self.view.areas.get("answer").unwrap(),
            self.view.name_selected("answer"),
        );
    }
}
