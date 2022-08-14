#![allow(non_camel_case_types)]


use crossterm::event::KeyCode;
use crate::utils::{
    card::{Card, Review, RecallGrade},
    sql::{
        fetch::load_cards,
        insert::revlog_new,
    },
    interval,
};

use rusqlite::Connection;


#[derive(PartialEq)]
pub enum ReviewSelection{
    question,
    answer,
    stats,
    dependents,
    dependencies,
}

pub struct ReviewList{
    pub title: String,
    pub cards: Vec<u32>,
    pub card: Option<u32>,
    pub reveal: bool,
    pub start_qty: u16,
    pub selection: ReviewSelection,
}


impl ReviewList {
    pub fn new(conn: &Connection)->ReviewList{
        interval::calc_strength(conn);
        let thecards = load_cards(conn).unwrap();

        let mut filtered = Vec::<u32>::new();
        for card in thecards{
            if card.strength < 0.99999{
                filtered.push(card.card_id);
            }
        }

        let qty = *(&filtered.len()) as u16;
        let thecard;

        if qty  > 0{
            thecard = Some(filtered[0 as usize]);
        }
        else {
            thecard = None;
        }


        ReviewList{
            title: String::from("reviewww"),
            cards: filtered,
            card: thecard,
            reveal: false,
            start_qty: qty,
            selection: ReviewSelection::question,
        }
    }
    pub fn new_review(&mut self, conn: &Connection, card: Option<Card>, grade: RecallGrade){
        if let None = card{
            return;
        }

        let mut card = card.unwrap();
        let review = Review::from(&grade);
        card.history.push(review.clone());
        revlog_new(conn, card.card_id, review).unwrap();
        interval::calc_stability(conn, &mut card);
        self.cards.pop();

        if self.cards.is_empty(){
            self.card = None;
        } else {
            self.card = Some(self.cards[self.cards.len() - 1 as usize].clone());
        }
        self.reveal = false;
    }


    pub fn navigate(&mut self, key: KeyCode){
        if key == KeyCode::Right && self.selection == ReviewSelection::question {self.selection = ReviewSelection::dependents}
        else if key == KeyCode::Down  && self.selection == ReviewSelection::question {self.selection = ReviewSelection::answer}

        else if key == KeyCode::Right && self.selection == ReviewSelection::answer   {self.selection = ReviewSelection::stats}
        else if key == KeyCode::Up    && self.selection == ReviewSelection::answer   {self.selection = ReviewSelection::question}
          
        else if key == KeyCode::Down && self.selection == ReviewSelection::dependents {self.selection = ReviewSelection::stats}
        else if key == KeyCode::Left && self.selection == ReviewSelection::dependents {self.selection = ReviewSelection::question}

        else if key == KeyCode::Left && self.selection == ReviewSelection::stats {self.selection = ReviewSelection::answer}
        else if key == KeyCode::Up   && self.selection == ReviewSelection::stats {self.selection = ReviewSelection::dependents}
        else if key == KeyCode::Down && self.selection == ReviewSelection::stats {self.selection = ReviewSelection::dependencies}

        else if key == KeyCode::Up   && self.selection == ReviewSelection::dependencies {self.selection = ReviewSelection::stats}
        else if key == KeyCode::Left && self.selection == ReviewSelection::dependencies {self.selection = ReviewSelection::answer}
    }
}
