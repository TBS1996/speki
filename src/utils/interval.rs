use crate::utils::{
    card::{RecallGrade, Review},
    sql::{
        update::{update_strength},
        fetch::{load_cards, get_history, get_stability},
    }    
};

use crate::utils::aliases::*;
use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::Connection;

use super::sql::{fetch::fetch_card, update::set_stability};



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
        let history = get_history(conn, card.id).unwrap();
        if card.is_complete() {
            let hislen = history.len();
            if hislen == 0 as usize{
                panic!{"wtf {}", &card.question};
            }
            let stability = get_stability(conn, card.id).unwrap() as f32;
            passed = time_passed_since_review(&history[(history.len() - 1) as usize]);
            strength = func(passed, stability);
            update_strength(conn, card.id, strength).unwrap();
        }
    }
}



pub fn calc_stability(conn: &Connection, id: CardID){
    let mut card = fetch_card(conn, id);
    let history = get_history(conn, id).unwrap();
    let hislen = history.len();
    
    if hislen < 2{
        panic!("too small {}, {}", hislen, card.question.clone());
    }

    let prev_stability = get_stability(conn, id).unwrap() as f32;
    let grade          =  &history[hislen - 1 as usize].grade;
    let time_passed    = time_passed_since_review(&history[(hislen - 2) as usize]);
    
    let gradefactor = match grade {
         RecallGrade::None   => 0.25,
         RecallGrade::Failed => 0.5,
         RecallGrade::Decent => 2.,
         RecallGrade::Easy   => 4.,
    };

    let new_stability = if gradefactor < 1.{
        time_passed * gradefactor
    } else {
        if time_passed > prev_stability{
            time_passed * gradefactor
        } else {
            ((prev_stability * gradefactor) - gradefactor) * time_passed/prev_stability + prev_stability
        }
    };

    set_stability(conn, card.id, new_stability).unwrap();
}


