use crate::utils::{
    card::{Card, RecallGrade, Review},
    sql::{
        update::{update_strength, update_stability},
        fetch::load_cards,
    }    
};

use std::time::{SystemTime, UNIX_EPOCH};



fn func(passed: f32, stability: f32) -> f32 {
    let E = 2.7182_f32;
    E.powf((0.9_f32).log(E) * passed/stability)
}

fn time_passed_since_review(review: &Review) -> f32 { 
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f32;
    let date = review.date as f32;
    let diff = now - date;
    
    diff / 86400_f32
}


pub fn calc_strength() {
    let cards = load_cards().unwrap();

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
            update_strength(&card, strength);
        }
    }
}



pub fn calc_stability(mut card: &mut Card){
    let hislen         = (&card.history).len();
    
    if hislen < 2{
        panic!("too small {}, {}", hislen, card.question.clone());
    }

    let prev_stability =   card.stability.clone();
    let grade          =  &card.history[hislen - 1 as usize].grade;
    let time_passed    = time_passed_since_review(&card.history[(hislen - 2) as usize]);

    let tp2;
    
    if hislen > 2{
        tp2 = time_passed_since_review(&card.history[hislen - 3]);
    }
    else {
        tp2 = time_passed;
    }
    
    let gradefactor;

    match grade {
         RecallGrade::None   => gradefactor = 0.25,
         RecallGrade::Failed => gradefactor = 0.5,
         RecallGrade::Decent => gradefactor = 2.,
         RecallGrade::Easy   => gradefactor = 4.,
         _ => panic!("incorrect enum!!:  {}", "oh shit")
    }

    let mut new_stability;

    if time_passed < tp2 {
        new_stability = prev_stability * (gradefactor * time_passed / tp2);
    }
    else {
        new_stability = gradefactor * time_passed;
    } 
    
    if new_stability < 0.001{
        new_stability = 0.001;
    }
    card.stability = new_stability;

    update_stability(card.clone());
}


