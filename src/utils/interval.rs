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

pub fn calc_stability(conn: &Arc<Mutex<Connection>>, id: CardID) {
    let history = get_history(conn, id);
    let hislen = history.len();
    let grade = &history[hislen - 1].grade;

    let gradefactor: f32 = match grade {
        RecallGrade::None => 0.25,
        RecallGrade::Failed => 0.5,
        RecallGrade::Decent => 2.,
        RecallGrade::Easy => 4.,
    };

    if hislen < 2 {
        set_stability(conn, id, Duration::from_secs_f32(gradefactor * 86400.));
        return;
    }

    let prev_stability = get_stability(conn, id).as_secs() as f32 / 86400.;
    let time_passed =
        time_passed_since_review(&history[(hislen - 2) as usize]).as_secs() as f32 / 86400.;

    let new_stability = if gradefactor < 1. {
        time_passed * gradefactor
    } else {
        if time_passed > prev_stability {
            time_passed * gradefactor
        } else {
            ((prev_stability * gradefactor) - gradefactor) * time_passed / prev_stability
                + prev_stability
        }
    };

    set_stability(conn, id, Duration::from_secs_f32(new_stability));
}
