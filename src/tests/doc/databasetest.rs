use std::{
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use rusqlite::Connection;

#[cfg(test)]
use crate::utils::misc::SpekiPaths;
use crate::utils::{
    aliases::Conn,
    card::{self, Card, CardType, CardTypeData, FinishedInfo, RecallGrade, Review, UnfinishedInfo},
    interval::{calc_stability, strength_algo},
    sql::{fetch::cards::fetch_card, init_db},
};
fn get_paths() -> SpekiPaths {
    let home = home::home_dir().unwrap();
    let mut paths = SpekiPaths::new(&home);
    paths.database = PathBuf::from_str("/home/tor/.local/share/speki/dbtest.db").unwrap();
    paths
}

fn get_conn() -> Arc<Mutex<Connection>> {
    let paths = get_paths();
    Arc::new(Mutex::new(
        Connection::open(&paths.database).expect("Failed to connect to database."),
    ))
}

#[test]
fn initdbtest() {
    let paths = get_paths();
    init_db(&paths.database).unwrap();
}

#[test]
fn strength_test() {
    let x = strength_algo(
        std::time::Duration::from_secs(86400),
        std::time::Duration::from_secs(86400),
    );
    assert_eq!(x, 0.9);
}

#[test]
fn new_stability_test() {
    let reviewvec = vec![Review {
        grade: RecallGrade::Easy,
        date: Duration::from_secs(1000),
        answertime: 0.,
    }];

    let new_review = Review {
        grade: RecallGrade::Easy,
        date: Duration::from_secs(2000),
        answertime: 0.,
    };

    let stability = Duration::from_secs(1000);

    let new_stab = calc_stability(&reviewvec, &new_review, stability);

    assert_eq!(new_stab, stability.mul_f32(RecallGrade::Easy.get_factor()));
}

fn new_half_correct_stability_test() {
    let reviewvec = vec![Review {
        grade: RecallGrade::Easy,
        date: Duration::from_secs(1000),
        answertime: 0.,
    }];

    let new_review = Review {
        grade: RecallGrade::Easy,
        date: Duration::from_secs(2000),
        answertime: 0.,
    };

    let factor = RecallGrade::Easy.get_factor();
    assert_eq!(factor, 2.);

    let stability = Duration::from_secs(500);

    let new_stab = calc_stability(&reviewvec, &new_review, stability);

    assert!(
        new_stab > Duration::from_secs(1000)
            && new_stab < Duration::from_secs(2000).mul_f32(factor)
    );
}

#[test]
fn dependency_logic() {
    initdbtest();
    let question = "What is the capital of france?".to_string();
    let answer = "paris".to_string();
    let conn = get_conn();

    let id1 = Card::new(CardTypeData::Finished(FinishedInfo::default()))
        .question(question.clone())
        .answer(answer.clone())
        .save_card(&conn);
    let card = fetch_card(&conn, id1);
    assert_eq!(question.clone(), card.question.clone());
    assert_eq!(answer.clone(), card.answer.clone());
    assert!(Card::is_resolved(&conn, id1));
    let id2 = Card::new(CardTypeData::Unfinished(UnfinishedInfo::default()))
        .question("what is a capital".to_string())
        .answer("the city from where the government rules".to_string())
        .dependents([id1])
        .save_card(&conn);
    assert!(!Card::is_resolved(&conn, id1));
    Card::complete_card(&conn, id2);
    assert!(Card::is_resolved(&conn, id1));

    let id3 = Card::new(CardTypeData::Unfinished(UnfinishedInfo::default()))
        .question("What is a government?".to_string())
        .answer("some people ruling the area".to_string())
        .dependents([id2])
        .save_card(&conn);

    assert!(!Card::is_resolved(&conn, id1));
    assert!(!Card::is_resolved(&conn, id2));

    Card::complete_card(&conn, id3);

    assert!(Card::is_resolved(&conn, id1));
    assert!(Card::is_resolved(&conn, id2));
}
