use crate::utils::card::Status;
use crate::utils::{aliases::*, card::Card};
use rusqlite::Connection;
use crate::utils::sql::fetch::{fetch_question, get_topic_of_card};
use crate::utils::widgets::textinput::Field;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction::Vertical, Layout, Rect},
    Frame,
};

use crate::utils::sql::fetch::{load_inc_text, get_topic_of_inc};
use crate::MyKey;
use crate::utils::misc::PopUpStatus;
use crate::Direction;

pub enum Selection{
    Question,
    Answer,
}

pub enum Purpose{
    Source(TopicID),
    Dependency(CardID),
    Dependent(CardID),
}

pub struct AddChildWidget{
    pub prompt: Field,
    pub question: Field,
    pub answer: Field,
    pub status: PopUpStatus,
    pub selection: Selection,
    pub purpose: Purpose,
}

impl AddChildWidget{
    pub fn new(conn: &Connection, purpose: Purpose) -> Self{
        let prompt = Self::add_prompt(conn, &purpose);
        let question = Field::new();
        let answer = Field::new();
        let selection = Selection::Question;
        let status = PopUpStatus::OnGoing;


        AddChildWidget{
            prompt,
            question,
            answer,
            status,
            selection,
            purpose,
        }
    }

    fn add_prompt(conn: &Connection, purpose: &Purpose) -> Field{
        let mut prompt = Field::new();
        match purpose{
            Purpose::Source(id) => {
                prompt.push("Add new sourced card".to_string());
                let sourcetext = load_inc_text(conn, *id).unwrap();
                prompt.push(sourcetext);
            },
            Purpose::Dependency(id) => {
                prompt.push("Add new dependent of: ".to_string());
                let ques = fetch_question(conn, *id);
                prompt.push(ques)
            },
            Purpose::Dependent(id) => {
                prompt.push("Add new dependency of: ".to_string());
                let ques = fetch_question(conn, *id);
                prompt.push(ques)
            },
        }
        prompt
    }

    pub fn keyhandler(&mut self, conn: &Connection, key: MyKey){
        use MyKey::*;
        match key {
            Esc => self.status = PopUpStatus::Finished,
            Alt('f') => self.submit_card(conn, true),
            Alt('u') => self.submit_card(conn, false),
            Nav(Direction::Up) => self.selection = Selection::Question,
            Nav(Direction::Down) => self.selection = Selection::Answer,
            key => {
                match self.selection{
                    Selection::Question => self.question.keyhandler(key),
                    Selection::Answer   => self.answer.keyhandler(key),
                }
            }
        }
    }

    fn submit_card(&mut self, conn: &Connection, isfinished: bool){
        let topic = match self.purpose{
            Purpose::Source(id)     => get_topic_of_inc(conn, id).unwrap(),
            Purpose::Dependent(id)  => get_topic_of_card(conn, id),
            Purpose::Dependency(id) => get_topic_of_card(conn, id),
        };


        let question = self.question.return_text();
        let answer = self.answer.return_text();
        let source = if let Purpose::Source(id) = self.purpose {id} else {0};
        let status = if isfinished{
            Status::new_complete()
        } else {
            Status::new_incomplete()
        };
        


        let mut card = Card::new()
            .question(question)
            .answer(answer)
            .topic(topic)
            .source(source)
            .status(status);

        match self.purpose{
            Purpose::Dependent(cid)  => {card.dependent (cid);},
            Purpose::Dependency(cid) => {card.dependency(cid);}
            _ => {},

        }
        card.save_card(conn);
        self.status = PopUpStatus::Finished;
    }


pub fn render<B>(&self, f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Vertical)
        .constraints([
                     Constraint::Ratio(2, 10),
                     Constraint::Ratio(4, 10),
                     Constraint::Ratio(4, 10)
        ]
        .as_ref(),)
        .split(area);
    
    let (prompt, question, answer) = (chunks[0], chunks[1], chunks[2]);


    let (mut qsel, mut asel) = (false, false);

    match self.selection{
        Selection::Question => qsel = true,
        Selection::Answer => asel = true,
    }

    self.prompt.draw_field(f, prompt, false);
    self.question.draw_field(f, question, qsel);
    self.answer.draw_field(f, answer, asel);
}

}
