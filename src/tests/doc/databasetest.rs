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
    interval::strength_algo,
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
fn interval_test() {
    let conn = get_conn();
    let x = strength_algo(
        std::time::Duration::from_secs(86400),
        std::time::Duration::from_secs(86400),
    );
    assert_eq!(x, 0.9);

    let id = Card::new(CardTypeData::Finished(FinishedInfo::default()))
        .question("Wtf debug".to_string())
        .save_card(&conn);
    let newone = Review {
        grade: RecallGrade::Decent,
        date: Duration::from_secs(1000000),
        answertime: 0.,
    };
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
