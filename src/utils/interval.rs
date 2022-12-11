use crate::utils::{
    card::{RecallGrade, Review},
    sql::{
        fetch::cards::{get_history, get_stability},
        update::update_strength,
    },
};

use crate::utils::aliases::*;
use rusqlite::Connection;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{
    card::CardType,
    misc::get_current_unix,
    sql::{fetch::CardQuery, update::set_stability},
};

use std::sync::{Arc, Mutex};

pub fn strength_algo(passed: Duration, stability: Duration) -> f32 {
    let e = std::f32::consts::E;
    (stability.as_secs() as f32 / passed.as_secs() as f32) * e.powf((0.9_f32).log(e))
}

fn time_passed_since_review(review: &Review) -> std::time::Duration {
    let now = get_current_unix();
    let date = review.date;
    now - date
}

pub fn calc_strength(conn: &Arc<Mutex<Connection>>) {
    let cards = CardQuery::default()
        .cardtype(vec![CardType::Finished])
        .fetch_card(conn);

    let mut strength;
    let mut passed;

    for card in cards.iter() {
        let history = get_history(conn, card.id);
        if card.is_complete() {
            let hislen = history.len();
            if hislen == 0 {
                panic! {"wtf {}", &card.question};
            }
            let stability = get_stability(conn, card.id);
            passed = time_passed_since_review(&history[(history.len() - 1) as usize]);
            strength = strength_algo(passed, stability);
            update_strength(conn, card.id, strength);
        }
    }
}

fn get_elapsed_time_reviews(reviews: &Vec<Review>) -> Vec<Duration> {
    let mut times = vec![];
    let revlen = reviews.len();
    assert!(revlen > 1);

    let mut prev_unix = reviews[0].date;
    let mut curr_unix;

    for idx in 1..revlen {
        curr_unix = reviews[idx].date;
        times.push(curr_unix - prev_unix);
        prev_unix = curr_unix;
    }
    times
}

pub fn calc_stability(
    history: &Vec<Review>,
    new_review: &Review,
    prev_stability: Duration,
) -> Duration {
    let gradefactor = new_review.grade.get_factor();
    if history.is_empty() {
        return Duration::from_secs_f32(gradefactor * 86400.);
    }

    let mut newstory;

    let timevec = get_elapsed_time_reviews({
        newstory = history.clone();
        newstory.push(new_review.clone());
        &newstory
    });
    let time_passed = timevec.last().unwrap();

    if gradefactor < 1. {
        return std::cmp::min(time_passed, &prev_stability).mul_f32(gradefactor);
    }

    if time_passed > &prev_stability {
        return time_passed.mul_f32(gradefactor);
    } else {
        let base = prev_stability;
        let max = base.mul_f32(gradefactor);
        let diff = max - base;

        let percentage = time_passed.div_f32(prev_stability.as_secs_f32());

        return (diff.mul_f32(percentage.as_secs_f32())) + base;
    }
}

// call function BEFORE you insert new review to database
pub fn new_card_stability(conn: Conn, id: CardID, new_review: &Review) -> Duration {
    let history = get_history(conn, id);
    let prev_stability = get_stability(conn, id);
    let new_stability = calc_stability(&history, &new_review, prev_stability);
    set_stability(conn, id, new_stability);
    new_stability
}
