#![allow(non_camel_case_types)]
use crate::app::App;
use crate::utils::{
    card::{Card, Review, RecallGrade},
    sql::{
        fetch::{load_cards, prev_id},
        insert::revlog_new,
    },
    interval,
};


pub struct ReviewList{
    pub title: String,
    pub cards: Vec<u32>,
    pub card: Option<u32>,
    pub reveal: bool,
    pub start_qty: u16,
}


impl ReviewList {
    pub fn new()->ReviewList{
        interval::calc_strength();
        let thecards = load_cards().unwrap();

        let mut filtered = Vec::<u32>::new();
        for card in thecards{
            if card.strength < 0.98{
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
        }
    }
    pub fn new_review(&mut self, card: Option<Card>, grade: RecallGrade){
        if let None = card{
            return;
        }

        let mut card = card.unwrap();
        let review = Review::from(&grade);
        card.history.push(review.clone());
        revlog_new(card.card_id, review);
        interval::calc_stability(&mut card);
        self.cards.pop();

        if self.cards.is_empty(){
            self.card = None;
        } else {
            self.card = Some(self.cards[self.cards.len() - 1 as usize].clone());
        }
        self.reveal = false;
    
    }
}
