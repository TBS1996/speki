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

pub fn get_history(conn: Conn, id: u32) -> PrettyResult<Vec<Review>> {
    let mut vecofrows = Vec::<Review>::new();
    conn.lock()
        .unwrap()
        .prepare("SELECT * FROM revlog WHERE cid = ? ORDER BY unix ASC")?
        .query_map([id], |row| {
            vecofrows.push(Review {
                grade: RecallGrade::from(row.get(2)?).unwrap(),
                date: std::time::Duration::from_secs(row.get(0)?),
                answertime: row.get(3)?,
            });
            Ok(())
        })?
        .for_each(|_| {});
    Ok(vecofrows)
}

fn get_pending_position(conn: Conn, id: CardID) -> u32 {
    conn.lock()
        .unwrap()
        .prepare("SELECT position FROM pending_cards where id = ?")
        .unwrap()
        .query_row([id], |row| Ok(row.get(0).unwrap()))
        .unwrap()
}

pub fn get_cardtypedata(row: &Row) -> CardTypeData {
    let id = row.get::<usize, u32>(0).unwrap();
    let cardtypedata = match row.get::<usize, u32>(7).unwrap() {
        0 => CardTypeData::Pending(PendingInfo { pos: 0 }),
        1 => CardTypeData::Unfinished(UnfinishedInfo::default()),
        2 => CardTypeData::Finished(FinishedInfo::default()),
        _ => panic!(),
    };
    todo!();
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
    let mut cardvec = Vec::<Card>::new();
    conn.lock()
        .unwrap()
        .prepare("SELECT * FROM cards")?
        .query_map([], |row| {
            cardvec.push(row2card(row));
            Ok(())
        })?
        .for_each(|_| {});
    for i in 0..cardvec.len() {
        let id = cardvec[i].id;
        cardvec[i].dependencies = get_dependencies(conn, id).unwrap();
        cardvec[i].dependents = get_dependents(conn, id).unwrap();
        cardvec[i].history = get_history(conn, id).unwrap();
    }

    Ok(cardvec)
}

pub fn load_card_matches(conn: Conn, search: &str) -> PrettyResult<Vec<Card>> {
    let mut cardvec = Vec::<Card>::new();
    conn
        .lock()
        .unwrap()
        .prepare("SELECT * FROM cards WHERE (question LIKE '%' || ?1 || '%') OR (answer LIKE '%' || ?1 || '%') LIMIT 50")?
        .query_map([search], |row| {
            cardvec.push(row2card(row));
            Ok(())
        })?.for_each(|_|{});
    for i in 0..cardvec.len() {
        let id = cardvec[i].id;
        cardvec[i].dependencies = get_dependencies(conn, id).unwrap();
        cardvec[i].dependents = get_dependents(conn, id).unwrap();
    }
    Ok(cardvec)
}

pub fn get_dependents(conn: Conn, dependency: u32) -> PrettyResult<Vec<u32>> {
    let mut depvec = Vec::<CardID>::new();
    conn.lock()
        .unwrap()
        .prepare("SELECT dependent FROM dependencies where dependency = ?")?
        .query_map([dependency], |row| {
            depvec.push(row.get(0).unwrap());
            Ok(())
        })?
        .for_each(|_| {});
    Ok(depvec)
}

pub fn get_dependencies(conn: Conn, dependent: CardID) -> PrettyResult<Vec<CardID>> {
    let mut depvec = Vec::<CardID>::new();
    conn.lock()
        .unwrap()
        .prepare("SELECT dependency FROM dependencies where dependent = ?")?
        .query_map([dependent], |row| {
            depvec.push(row.get(0).unwrap());
            Ok(())
        })?
        .for_each(|_| {});
    Ok(depvec)
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
    conn.lock()
        .unwrap()
        .query_row("select resolved FROM cards WHERE id=?", [id], |row| {
            row.get(0)
        })
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
    conn.lock()
        .unwrap()
        .query_row("select skipduration FROM incread WHERE id=?", [id], |row| {
            row.get(0)
        })
        .unwrap()
}

pub fn get_skiptime(conn: Conn, id: CardID) -> u32 {
    conn.lock()
        .unwrap()
        .query_row(
            "select skiptime FROM unfinished_cards WHERE id=?",
            [id],
            |row| row.get(0),
        )
        .unwrap()
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
    card.dependents = get_dependents(conn, card.id).unwrap();
    card.dependencies = get_dependencies(conn, card.id).unwrap();
    card
}

pub fn fetch_card(conn: Conn, cid: u32) -> Card {
    let card = conn
        .lock()
        .unwrap()
        .query_row("SELECT * FROM cards WHERE id=?", [cid], |row| {
            Ok(row2card(row))
        })
        .expect(&format!("Failed to query following card: {}", cid));
    fill_dependencies(conn, card)
}

pub fn get_pending_qty(conn: Conn) -> u32 {
    conn.lock()
        .unwrap()
        .query_row("SELECT COUNT(*) FROM pending_cards", [], |row| {
            Ok(row.get(0).unwrap())
        })
        .unwrap()
}

pub fn fill_card_vec(cardvec: &mut Vec<Card>, conn: Conn) {
    for i in 0..cardvec.len() {
        let id = cardvec[i].id;
        cardvec[i].dependencies = get_dependencies(conn, id).unwrap();
        cardvec[i].dependents = get_dependents(conn, id).unwrap();
        cardvec[i].history = get_history(conn, id).unwrap();
    }
}

pub fn get_highest_pos(conn: Conn) -> Option<u32> {
    if is_table_empty(conn, "pending_cards".to_string()) {
        return None;
    }
    let highest = conn
        .lock()
        .unwrap()
        .query_row("SELECT MAX(position) FROM pending_cards", [], |row| {
            Ok(row.get(0).unwrap())
        })
        .unwrap();
    Some(highest)
}
