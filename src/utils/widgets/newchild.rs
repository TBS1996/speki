use crate::utils::card;
use crate::utils::{aliases::*, sql::insert::update_both, card::Card};
use rusqlite::Connection;
use crate::utils::sql::fetch::load_cards;
use crate::utils::statelist::StatefulList;
use crate::utils::widgets::textinput::Field;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction::Vertical, Layout, Rect},
    Frame,
};

use crate::utils::sql::fetch::{load_inc_text, get_topic_of_inc};
use crate::utils::widgets::list::list_widget;
use super::message_box::draw_message;
use crate::MyKey;
use crate::utils::misc::PopUpStatus;
use crate::Direction;

pub enum Selection{
    Question,
    Answer,
}


pub struct AddChildWidget{
    pub prompt: Field,
    pub id: TopicID,
    pub question: Field,
    pub answer: Field,
    pub status: PopUpStatus,
    pub selection: Selection,
}

impl AddChildWidget{
    pub fn new(conn: &Connection, id: TopicID) -> Self{
        let question = Field::new();
        let answer = Field::new();
        let selection = Selection::Question;
        let sourcetext = load_inc_text(conn, id).unwrap();
        let mut prompt = Field::new();
        prompt.replace_text(sourcetext);
        let status = PopUpStatus::OnGoing;

        AddChildWidget{
            prompt,
            id,
            question,
            answer,
            status,
            selection,

        }
    }

    pub fn keyhandler(&mut self, conn: &Connection, key: MyKey){
        match key {
            MyKey::Esc => self.status = PopUpStatus::Finished,
            MyKey::Alt('f') => self.submit_card(conn, true),
            MyKey::Alt('u') => self.submit_card(conn, false),
            MyKey::Up   => self.selection = Selection::Question,
            MyKey::Down => self.selection = Selection::Answer,
            MyKey::Nav(Direction::Up) => self.selection = Selection::Question,
            MyKey::Nav(Direction::Down) => self.selection = Selection::Answer,
            key => {
                match self.selection{
                    Selection::Question => self.question.keyhandler(key),
                    Selection::Answer   => self.answer.keyhandler(key),
                }
            }
        }
    }

    fn submit_card(&mut self, conn: &Connection, isfinished: bool){
        let topic = get_topic_of_inc(conn, self.id).unwrap();
        let question = self.question.return_text();
        let answer = self.answer.return_text();
        Card::save_new_card(conn, question, answer, topic, self.id, isfinished);
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
