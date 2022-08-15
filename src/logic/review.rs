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
    Question,
    Answer,
    Stats,
    Dependents,
    Dependencies,
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
            selection: ReviewSelection::Question,
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
        if key == KeyCode::Right && self.selection == ReviewSelection::Question {self.selection = ReviewSelection::Dependents}
        else if key == KeyCode::Down  && self.selection == ReviewSelection::Question {self.selection = ReviewSelection::Answer}

        else if key == KeyCode::Right && self.selection == ReviewSelection::Answer   {self.selection = ReviewSelection::Stats}
        else if key == KeyCode::Up    && self.selection == ReviewSelection::Answer   {self.selection = ReviewSelection::Question}
          
        else if key == KeyCode::Down && self.selection == ReviewSelection::Dependents {self.selection = ReviewSelection::Stats}
        else if key == KeyCode::Left && self.selection == ReviewSelection::Dependents {self.selection = ReviewSelection::Question}

        else if key == KeyCode::Left && self.selection == ReviewSelection::Stats {self.selection = ReviewSelection::Answer}
        else if key == KeyCode::Up   && self.selection == ReviewSelection::Stats {self.selection = ReviewSelection::Dependents}
        else if key == KeyCode::Down && self.selection == ReviewSelection::Stats {self.selection = ReviewSelection::Dependencies}

        else if key == KeyCode::Up   && self.selection == ReviewSelection::Dependencies {self.selection = ReviewSelection::Stats}
        else if key == KeyCode::Left && self.selection == ReviewSelection::Dependencies {self.selection = ReviewSelection::Answer}
    }
}
