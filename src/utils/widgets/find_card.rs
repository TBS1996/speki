use rusqlite::Connection;
use crate::utils::sql::fetch::load_cards;
use crate::utils::statelist::StatefulList;
use crate::utils::widgets::textinput::{Field, draw_field};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction::{Vertical, Horizontal}, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{
        Block, Borders, Gauge},
    Frame,
};

use super::message_box::draw_message;
use crossterm::event::KeyCode;

#[derive(Clone, PartialEq)]
pub struct CardMatch{
    pub question: String,
    pub id: u32,
}

#[derive(Clone, PartialEq)]
pub struct FindCardWidget{
    pub prompt: String,
    pub searchterm: Field,
    pub list: StatefulList<CardMatch>,
    pub chosen_card: Option<u32>,
    pub exit: bool,
}


impl FindCardWidget{
    pub fn new(conn: &Connection, prompt: String) -> Self{
        let mut list = StatefulList::<CardMatch>::new();
        let searchterm = Field::new();
        list.reset_filter(conn, searchterm.text.clone());
        let chosen_card = None;
        let exit = false;
        FindCardWidget{
            prompt,
            searchterm,
            list,
            chosen_card,
            exit,
        }
    }

    pub fn keyhandler(&mut self, conn: &Connection, key: KeyCode){
        if key == KeyCode::Enter{
            if self.list.state.selected().is_some(){
                self.chosen_card = Some(self.list.items[self.list.state.selected().unwrap()].id);
                self.exit = true;
            }
        }
        if key == KeyCode::Esc{self.exit = true;}


        self.searchterm.keyhandler(key);
        self.list.reset_filter(conn, self.searchterm.text.clone());

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

use crate::utils::widgets::list::list_widget;




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
    draw_field(f, searchbar, vec![Span::from(widget.searchterm.text.as_str())], "", Alignment::Left, false);
    list_widget(f, &widget.list, matchlist, false);
}












































