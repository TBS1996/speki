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
use crate::widgets::button::Button;
use crate::widgets::textinput::Field;
use crate::{MyKey, MyType};
use rusqlite::Connection;
use tui::layout::Rect;
use tui::Frame;

use crate::widgets::topics::TopicList;

//#[derive(Clone)]
pub struct NewCard {
    pub prompt: Button,
    pub question: Field,
    pub answer: Field,
    pub topics: TopicList,
    view: View,
    area: Rect,
}

use std::sync::{Arc, Mutex};

impl NewCard {
    pub fn new(conn: &Arc<Mutex<Connection>>) -> NewCard {
        let topics = TopicList::new(conn);
        let view = View::default();

        NewCard {
            prompt: Button::new("Add new card".to_string()),
            question: Field::new("Question".to_string()),
            answer: Field::new("Answer".to_string()),
            topics,
            view,
            area: Rect::default(),
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

    fn get_view(&mut self) -> &mut View {
        &mut self.view
    }

    fn set_selection(&mut self, area: Rect) {
        self.view.areas.clear();
        let chunks = split_leftright_by_percent([75, 15], area);
        let left = chunks[0];
        let right = chunks[1];
        let chunks = split_updown_by_percent([10, 37, 37], left);

        self.view.areas.clear();
        self.view.areas.push(chunks[0]);
        self.view.areas.push(right);
        self.view.areas.push(chunks[1]);
        self.view.areas.push(chunks[2]);

        self.prompt.set_area(chunks[0]);
        self.question.set_area(chunks[1]);
        self.answer.set_area(chunks[2]);
        self.topics.set_area(right);
    }

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        use MyKey::*;
        let cursor = &self.get_cursor();

        match key {
            Alt('f') => self.submit_card(&appdata.conn, true),
            Alt('u') => self.submit_card(&appdata.conn, false),
            Alt('g') => {
                if let Some(key) = &appdata.config.gptkey {
                    let answer = get_gpt3_response(key, &self.question.return_text());
                    self.answer.replace_text(answer);
                }
            }
            key if self.question.is_selected(cursor) => self.question.keyhandler(appdata, key),
            key if self.answer.is_selected(cursor) => self.answer.keyhandler(appdata, key),
            key if self.topics.is_selected(cursor) => self.topics.keyhandler(appdata, key),
            _ => {}
        }
    }
    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, _area: Rect) {
        let cursor = &self.get_cursor();

        self.topics.render(f, appdata, cursor);
        self.prompt.render(f, appdata, cursor);
        self.question.render(f, appdata, cursor);
        self.answer.render(f, appdata, cursor);
    }
}
