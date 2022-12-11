use std::{io::Error, path::PathBuf, time::Duration};

use rusqlite::{Row, Rows};

use color_eyre::eyre::Result as PrettyResult;

use crate::utils::{
    aliases::*,
    ankitemplate::MediaContents,
    card::{
        Card, CardType, CardTypeData, FinishedInfo, PendingInfo, RecallGrade, Review,
        UnfinishedInfo,
    },
};

use super::{fetch_item, fetch_items, is_table_empty};

pub fn get_history(conn: Conn, id: u32) -> Vec<Review> {
    fetch_items(
        conn,
        format!("SELECT * FROM revlog WHERE cid={} ORDER BY unix ASC", id),
        |row| Review {
            grade: RecallGrade::from(row.get(2).unwrap()).unwrap(),
            date: std::time::Duration::from_secs(row.get(0).unwrap()),
            answertime: row.get(3).unwrap(),
        },
    )
    .unwrap()
}

fn get_pending_position(conn: Conn, id: CardID) -> u32 {
    fetch_item(
        conn,
        format!("SELECT position FROM pending_cards WHERE id={}", id),
        |row| row.get::<usize, u32>(0),
    )
    .unwrap()
}

pub fn get_cardtypedata(conn: Conn, id: CardID) -> CardTypeData {
    let cardtype = get_cardtype(conn, id);

    let cardtypedata = match cardtype {
        CardType::Pending => CardTypeData::Pending(PendingInfo::default()),
        CardType::Unfinished => CardTypeData::Unfinished(UnfinishedInfo::default()),
        CardType::Finished => CardTypeData::Finished(FinishedInfo::default()),
        _ => panic!(),
    };
    match &cardtypedata {
        CardTypeData::Pending(_) => {
            let pos = get_pending_position(conn, id);
            CardTypeData::Pending(PendingInfo { pos })
        }
        CardTypeData::Unfinished(_) => {
            let skiptime = get_skiptime(conn, id);
            let skipduration = get_skipduration(conn, id);
            CardTypeData::Unfinished(UnfinishedInfo {
                skiptime,
                skipduration,
            })
        }
        CardTypeData::Finished(_) => {
            let strength = get_strength(conn, id);
            let stability = get_stability(conn, id);
            CardTypeData::Finished(FinishedInfo {
                strength,
                stability,
            })
        }
    }
}

pub fn row2card(row: &Row) -> Card {
    let cardtype = match row.get::<usize, u32>(7).unwrap() {
        0 => CardTypeData::Pending(PendingInfo::default()),
        1 => CardTypeData::Unfinished(UnfinishedInfo::default()),
        2 => CardTypeData::Finished(FinishedInfo::default()),
        _ => panic!(),
    };
    let id = row.get(0).unwrap();

    let frontaudio: Option<String> = row.get(3).unwrap();
    let backaudio: Option<String> = row.get(4).unwrap();
    let frontimage: Option<String> = row.get(5).unwrap();
    let backimage: Option<String> = row.get(6).unwrap();

    let frontaudio: Option<PathBuf> = frontaudio.map(|x| PathBuf::from(x));
    let backaudio: Option<PathBuf> = backaudio.map(|x| PathBuf::from(x));
    let frontimage: Option<PathBuf> = frontimage.map(|x| PathBuf::from(x));
    let backimage: Option<PathBuf> = backimage.map(|x| PathBuf::from(x));

    //  let dependencies = get_dependencies(conn, id).unwrap();
    //  let dependents = get_depndents(conn, id).unwrap();
    Card {
        id,
        question: row.get(1).unwrap(),
        answer: row.get(2).unwrap(),
        frontaudio,
        backaudio,
        frontimage,
        backimage,
        cardtype,
        suspended: row.get(8).unwrap(),
        resolved: row.get(9).unwrap(),
        dependents: Vec::new(),
        dependencies: Vec::new(),
        history: Vec::new(),
        topic: row.get(10).unwrap(),
        source: row.get(11).unwrap(),
    }
}

pub fn load_cards(conn: Conn) -> PrettyResult<Vec<Card>> {
    let mut cardvec =
        fetch_items(conn, "Select * FROM cards".to_string(), |row| row2card(row)).unwrap();

    for i in 0..cardvec.len() {
        let id = cardvec[i].id;
        cardvec[i].dependencies = get_dependencies(conn, id);
        cardvec[i].dependents = get_dependents(conn, id);
        cardvec[i].history = get_history(conn, id);
    }

    Ok(cardvec)
}

pub fn load_card_matches(conn: Conn, search: &str) -> PrettyResult<Vec<Card>> {
    let mut cardvec = fetch_items(
        conn,
        format!("SELECT * FROM cards WHERE (question LIKE '%' || {} || '%') OR (answer LIKE '%' || {} || '%') LIMIT 50", search, search),
        |row| row2card(row)
        ).unwrap();

    for i in 0..cardvec.len() {
        let id = cardvec[i].id;
        cardvec[i].dependencies = get_dependencies(conn, id);
        cardvec[i].dependents = get_dependents(conn, id);
    }
    Ok(cardvec)
}

pub fn get_dependents(conn: Conn, dependency: u32) -> Vec<CardID> {
    fetch_items(
        conn,
        format!(
            "SELECT dependent FROM dependencies WHERE dependency = {}",
            dependency
        ),
        |row| row.get::<usize, CardID>(0).unwrap(),
    )
    .unwrap()
}

pub fn get_dependencies(conn: Conn, dependent: u32) -> Vec<CardID> {
    fetch_items(
        conn,
        format!(
            "SELECT dependency FROM dependencies WHERE dependent = {}",
            dependent
        ),
        |row| row.get::<usize, CardID>(0).unwrap(),
    )
    .unwrap()
}

//use crate::utils::card::CardType;
pub fn get_cardtype(conn: Conn, id: CardID) -> CardType {
    match conn
        .lock()
        .unwrap()
        .query_row("select cardtype FROM cards WHERE id=?", [id], |row| {
            row.get::<usize, usize>(0)
        })
        .unwrap()
    {
        0 => CardType::Pending,
        1 => CardType::Unfinished,
        2 => CardType::Finished,
        _ => panic!(),
    }
}

pub fn is_pending(conn: Conn, id: CardID) -> bool {
    CardType::Pending == get_cardtype(conn, id)
}

pub fn is_resolved(conn: Conn, id: CardID) -> bool {
    fetch_item(
        conn,
        format!("SELECT resolved FROM cards WHERE id={}", id),
        |row| row.get(0),
    )
    .unwrap()
}

pub struct PendingItem {
    pub id: CardID,
    pub pos: u32,
}

pub fn get_pending(conn: Conn) -> Vec<PendingItem> {
    fetch_items(
        conn,
        "Select * from pending_cards ORDER BY position ASC".to_string(),
        |row| PendingItem {
            id: row.get(0).unwrap(),
            pos: row.get(1).unwrap(),
        },
    )
    .unwrap()
}

pub fn get_skipduration(conn: Conn, id: CardID) -> u32 {
    fetch_item(
        conn,
        format!("select skipduration FROM unfinished_cards WHERE id={}", id),
        |row| row.get(0),
    )
    .unwrap()
}

pub fn get_inc_skipduration(conn: Conn, id: IncID) -> u32 {
    fetch_item(
        conn,
        format!("SELECT skipduration FROM incread WHERE id={}", id),
        |row| row.get(0),
    )
    .unwrap()
}

pub fn get_skiptime(conn: Conn, id: CardID) -> Duration {
    let dur = fetch_item(
        conn,
        format!("SELECT skiptime FROM unfinished_cards WHERE id={}", id),
        |row| row.get(0),
    )
    .unwrap();

    Duration::from_secs(dur)
}

pub fn get_stability(conn: Conn, id: CardID) -> Duration {
    fetch_item(
        conn,
        format!("select stability FROM finished_cards WHERE id={}", id),
        |row| {
            Ok(Duration::from_secs_f64(
                row.get::<usize, f64>(0).unwrap() * 86400.,
            ))
        },
    )
    .expect(&format!(
        "Couldn't find stability of card with following id: {}",
        id
    ))
}

pub fn get_strength(conn: Conn, id: CardID) -> f32 {
    fetch_item(
        conn,
        format!("SELECT strength FROM finished_cards WHERE id={}", id),
        |row| row.get(0),
    )
    .unwrap()
}

pub fn fetch_media(conn: Conn, id: CardID) -> MediaContents {
    let card = fetch_card(conn, id);
    MediaContents {
        frontaudio: card.frontaudio,
        backaudio: card.backaudio,
        frontimage: card.frontimage,
        backimage: card.backimage,
    }
}

#[derive(Clone)]
pub struct DepPair {
    dependent: u32,
    dependency: u32,
}

pub fn prev_id(conn: Conn) -> Result<u32, Error> {
    Ok(conn.lock().unwrap().last_insert_rowid() as u32)
}

pub fn fetch_question(conn: Conn, cid: CardID) -> String {
    fetch_card(conn, cid).question
}

pub fn get_topic_of_card(conn: Conn, cid: CardID) -> TopicID {
    fetch_card(conn, cid).topic
}

pub fn fill_dependencies(conn: Conn, mut card: Card) -> Card {
    card.dependents = get_dependents(conn, card.id);
    card.dependencies = get_dependencies(conn, card.id);
    card
}

pub fn fetch_card(conn: Conn, cid: u32) -> Card {
    let card = fetch_item(
        conn,
        format!("SELECT * FROM cards WHERE id={}", cid),
        |row| Ok(row2card(row)),
    )
    .unwrap();

    fill_dependencies(conn, card)
}

pub fn get_pending_qty(conn: Conn) -> u32 {
    fetch_item(
        conn,
        "SELECT COUNT(*) FROM pending_cards".to_string(),
        |row| row.get(0),
    )
    .unwrap()
}
pub fn fill_card_vec(cardvec: &mut Vec<Card>, conn: Conn) {
    for i in 0..cardvec.len() {
        let id = cardvec[i].id;
        cardvec[i].dependencies = get_dependencies(conn, id);
        cardvec[i].dependents = get_dependents(conn, id);
        cardvec[i].history = get_history(conn, id);
    }
}

pub fn get_highest_pos(conn: Conn) -> Option<u32> {
    if is_table_empty(conn, "pending_cards".to_string()) {
        return None;
    }
    let highest = fetch_item(
        conn,
        "SELECT MAX(position) FROM pending_cards".to_string(),
        |row| row.get(0),
    )
    .unwrap();
    Some(highest)
}
