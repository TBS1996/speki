use crossterm::event::KeyCode;
use crate::utils::{
    card::{Card, Review, RecallGrade},
    sql::{
        fetch::load_cards,
        insert::revlog_new,
    },
    interval, widgets::find_card::FindCardWidget,
};

use rusqlite::Connection;

#[derive(PartialEq)]
pub enum CardPurpose{
    Dependency,
    Dependent,
}

#[derive(PartialEq)]
pub struct SelectCard{
    pub cardfinder: FindCardWidget,
    pub purpose: CardPurpose,
}

impl SelectCard{
    pub fn new(conn: &Connection, prompt: String, purpose: CardPurpose) -> Self{
       SelectCard{
           cardfinder: FindCardWidget::new(conn, prompt),
           purpose,
       } 
    }
}


#[derive(PartialEq)]
pub enum ReviewSelection{
    Question,
    Answer,
    Stats,
    Dependents,
    Dependencies,
    SelectCard(SelectCard),
}

pub struct ReviewList{
    pub title: String,
    pub cards: Vec<u32>,
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


        ReviewList{
            title: String::from("reviewww"),
            cards: filtered,
            reveal: false,
            start_qty: qty,
            selection: ReviewSelection::Question,
        }
    }
    pub fn new_review(&mut self, conn: &Connection, card: Option<Card>, grade: RecallGrade){
        if self.cards.is_empty() {return}

        let mut card = card.unwrap();
        let review = Review::from(&grade);
        card.history.push(review.clone());
        revlog_new(conn, card.card_id, review).unwrap();
        interval::calc_stability(conn, &mut card);
        self.cards.remove(0 as usize);

        self.reveal = false;
    }

    pub fn select_card(&mut self, conn: &Connection, prompt: String, purpose: CardPurpose) {
        self.selection = ReviewSelection::SelectCard(SelectCard::new(conn, prompt, purpose));
    }


    pub fn navigate(&mut self, key: KeyCode){
        match (key, &self.selection){
            (KeyCode::Right, ReviewSelection::Question)     => {self.selection = ReviewSelection::Dependents},
            (KeyCode::Down,  ReviewSelection::Question)     => {self.selection = ReviewSelection::Answer},
            (KeyCode::Right, ReviewSelection::Answer)       => {self.selection = ReviewSelection::Dependencies},
            (KeyCode::Up,    ReviewSelection::Answer)       => {self.selection = ReviewSelection::Question},
            (KeyCode::Down,  ReviewSelection::Answer)       => {self.selection = ReviewSelection::Stats},
            (KeyCode::Down,  ReviewSelection::Dependents)   => {self.selection = ReviewSelection::Dependencies},
            (KeyCode::Left,  ReviewSelection::Dependents)   => {self.selection = ReviewSelection::Question},
            (KeyCode::Left,  ReviewSelection::Stats)        => {self.selection = ReviewSelection::Answer},
            (KeyCode::Up,    ReviewSelection::Stats)        => {self.selection = ReviewSelection::Answer},
            (KeyCode::Down,  ReviewSelection::Stats)        => {self.selection = ReviewSelection::Dependencies},
            (KeyCode::Up,    ReviewSelection::Dependencies) => {self.selection = ReviewSelection::Dependents},
            (KeyCode::Left,  ReviewSelection::Dependencies) => {self.selection = ReviewSelection::Answer},
            (KeyCode::Down,  ReviewSelection::Dependencies) => {self.selection = ReviewSelection::Stats},
            _ => {},
        }
    }
}
