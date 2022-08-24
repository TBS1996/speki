use crate::utils::{
    card::{Card, RecallGrade, Review},
    sql::{
        update::{update_strength, update_stability},
        fetch::load_cards,
    }    
};

use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::Connection;



fn func(passed: f32, stability: f32) -> f32 {
    let e = 2.7182_f32;
    e.powf((0.9_f32).log(e) * passed/stability)
}

fn time_passed_since_review(review: &Review) -> f32 { 
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f32;
    let date = review.date as f32;
    let diff = now - date;
    
    diff / 86400_f32
}


pub fn calc_strength(conn: &Connection) {
    let cards = load_cards(conn).unwrap();

    let mut strength;
    let mut passed;

    for card in cards.iter(){
        if card.status.isactive() {
            let hislen = card.history.len();
            if hislen == 0 as usize{
                panic!{"wtf {}", &card.question};
            }
            passed = time_passed_since_review(&card.history[(card.history.len() - 1) as usize]);
            strength = func(passed, card.stability);
            update_strength(conn, &card, strength).unwrap();
        }
    }
}



pub fn calc_stability(conn: &Connection, mut card: &mut Card){
    let hislen         = (&card.history).len();
    
    if hislen < 2{
        panic!("too small {}, {}", hislen, card.question.clone());
    }

    let prev_stability =   card.stability.clone();
    let grade          =  &card.history[hislen - 1 as usize].grade;
    let time_passed    = time_passed_since_review(&card.history[(hislen - 2) as usize]);

    
    let tp2 = if hislen > 2 {time_passed_since_review(&card.history[hislen - 3])} else {time_passed};
    let gradefactor = match grade {
         RecallGrade::None   => 0.25,
         RecallGrade::Failed => 0.5,
         RecallGrade::Decent => 2.,
         RecallGrade::Easy   => 4.,
    };

    
    let new_stability = if time_passed < tp2 {prev_stability * (gradefactor * time_passed / tp2)} else {gradefactor * time_passed};
    new_stability.min(0.0001);

    card.stability = new_stability;

    update_stability(conn, card.clone()).unwrap();
}


