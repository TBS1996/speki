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

use crate::utils::widgets::list::list_widget;
use super::message_box::draw_message;
use crossterm::event::KeyCode;



pub struct FindCardWidget{
    pub prompt: String,
    pub searchterm: Field,
    pub list: StatefulList<CardMatch>,
    pub status: FindCardStatus,
    pub purpose: CardPurpose,
}

pub enum FindCardStatus{
    Searching,
    Finished,
}

#[derive(Clone, PartialEq)]
pub struct CardMatch{
    pub question: String,
    pub id: CardID,
}

pub enum CardPurpose{
    NewDependency(CardID),
    NewDependent(CardID),
    NewCloze(TopicID),
}


impl FindCardWidget{
    pub fn new(conn: &Connection, prompt: String, purpose: CardPurpose) -> Self{
        let mut list = StatefulList::<CardMatch>::new();
        let searchterm = Field::new();
        list.reset_filter(conn, searchterm.text.clone());
        let status = FindCardStatus::Searching;

        FindCardWidget{
            prompt,
            searchterm,
            list,
            status,
            purpose,

        }
    }

    pub fn keyhandler(&mut self, conn: &Connection, key: KeyCode){
        match key {
            KeyCode::Enter => self.complete(conn), 
            KeyCode::Esc => self.status = FindCardStatus::Finished,
            KeyCode::Down => self.list.next(),
            KeyCode::Up => self.list.previous(),
            key => {
                self.searchterm.keyhandler(key);
                self.list.reset_filter(conn, self.searchterm.text.clone());
            }
        }
    }


    fn complete(&mut self, conn: &Connection){
        if self.list.state.selected().is_none() {return}

        let idx = self.list.state.selected().unwrap();
        let chosen_id = self.list.items[idx].id;

        match self.purpose{
            CardPurpose::NewDependent(source_id) => {
                update_both(conn, chosen_id, source_id).unwrap();
                Card::check_resolved(chosen_id, conn);
            },
            CardPurpose::NewDependency(source_id) => {
                update_both(conn, source_id, chosen_id).unwrap();
                Card::check_resolved(source_id, conn);
            },
            CardPurpose::NewCloze(_topic_id) => {
                todo!();
            }, 
        }
        self.status = FindCardStatus::Finished;
    }
}




impl StatefulList<CardMatch>{

pub fn reset_filter(&mut self, conn: &Connection, mut searchterm: String){
    let mut matching_cards = Vec::<CardMatch>::new();
    let all_cards = load_cards(conn).unwrap();
    searchterm.pop();
    for card in all_cards{
        if card.question.to_lowercase().contains(&searchterm) || card.answer.to_lowercase().contains(&searchterm){
            matching_cards.push(
                CardMatch{
                    question: card.question,
                    id: card.card_id,
                }
            );
        };
        }
        self.items = matching_cards;
    }


    pub fn choose_card(&self) -> u32{
        let index = self.state.selected().unwrap();
        self.items[index].id 
    }
}





pub fn draw_find_card<B>(f: &mut Frame<B>, widget: &FindCardWidget, area: Rect)
where
    B: Backend,
{



    let chunks = Layout::default()
        .direction(Vertical)
        .constraints([
                     Constraint::Ratio(1, 10),
                     Constraint::Ratio(1, 10),
                     Constraint::Ratio(8, 10)
        ]
        .as_ref(),)
        .split(area);
    
    let (prompt, searchbar, matchlist) = (chunks[0], chunks[1], chunks[2]);
    
    draw_message(f, prompt, &widget.prompt);
    widget.searchterm.draw_field(f, searchbar, "", Alignment::Left, false);
    list_widget(f, &widget.list, matchlist, false);
}












































