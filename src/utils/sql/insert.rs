use crate::utils::aliases::*;
use crate::utils::card::{Card, CardType, Review}; //, Status, Topic, Review}
use crate::utils::sql::update::set_cardtype;
use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn save_card(conn: &Arc<Mutex<Connection>>, card: Card) -> CardID {
    //let time_added = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
    let frontaudio: Option<String> = card
        .frontaudio
        .map(|x| x.into_os_string().into_string().unwrap());
    let backaudio: Option<String> = card
        .backaudio
        .map(|x| x.into_os_string().into_string().unwrap());
    let frontimage: Option<String> = card
        .frontimage
        .map(|x| x.into_os_string().into_string().unwrap());
    let backimage: Option<String> = card
        .backimage
        .map(|x| x.into_os_string().into_string().unwrap());

    let cardtype = match card.cardtype {
        CardType::Pending => 0,
        CardType::Unfinished => 1,
        CardType::Finished => 2,
    } as u32;

    conn.lock()
        .unwrap()
        .execute(
            "INSERT INTO cards (
            question, 
            answer, 
            frontaudio, 
            backaudio, 
            frontimg, 
            backimg, 
            cardtype, 
            suspended, 
            resolved, 
            topic, 
            source
            ) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                card.question,
                card.answer,
                frontaudio,
                backaudio,
                frontimage,
                backimage,
                cardtype,
                card.suspended,
                card.resolved,
                card.topic,
                card.source,
            ],
        )
        .unwrap();

    let id = conn.lock().unwrap().last_insert_rowid() as u32;

    match card.cardtype {
        CardType::Pending => new_pending(conn, id).unwrap(),
        CardType::Unfinished => new_unfinished(conn, id).unwrap(),
        CardType::Finished => new_finished(conn, id).unwrap(),
    };

    id
}

pub fn update_both(conn: &Arc<Mutex<Connection>>, dependent: u32, dependency: u32) -> Result<()> {
    conn.lock().unwrap().execute(
        "INSERT INTO dependencies (dependent, dependency) VALUES (?1, ?2)",
        params![dependent, dependency],
    )?;
    Ok(())
}

pub fn revlog_new(conn: &Arc<Mutex<Connection>>, card_id: u32, review: Review) -> Result<()> {
    conn.lock().unwrap().execute(
        "INSERT INTO revlog (unix, cid, grade, qtime, atime) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            review.date,
            card_id,
            review.grade as u32,
            review.answertime,
            -1
        ],
    )?;
    Ok(())
}

pub fn new_topic(conn: &Arc<Mutex<Connection>>, name: String, parent: u32, pos: u32) -> Result<()> {
    conn.lock().unwrap().execute(
        "INSERT INTO topics (name, parent, relpos) VALUES (?1, ?2, ?3)",
        params![name, parent, pos],
    )?;
    Ok(())
}

pub fn new_incread(
    conn: &Arc<Mutex<Connection>>,
    parent: u32,
    topic: u32,
    source: String,
    isactive: bool,
) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    conn.lock().unwrap().execute(
        "INSERT INTO incread (parent, topic, source, active, skiptime, skipduration, row, column) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![parent, topic, source, isactive, now, 1.0, 0, 0],
    )?;
    Ok(())
}

pub fn new_finished(conn: &Arc<Mutex<Connection>>, id: CardID) -> Result<()> {
    conn.lock().unwrap().execute(
        "INSERT INTO finished_cards (id, strength, stability) VALUES (?1, ?2, ?3)",
        params![id, 1.0f32, 1.0f32],
    )?;
    set_cardtype(conn, id, CardType::Finished).unwrap();
    Ok(())
}

pub fn new_unfinished(conn: &Arc<Mutex<Connection>>, id: CardID) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    conn.lock().unwrap().execute(
        "INSERT INTO unfinished_cards (id, skiptime, skipduration) VALUES (?1, ?2, ?3)",
        params![id, now, 1],
    )?;
    Ok(())
}
pub fn new_pending(conn: &Arc<Mutex<Connection>>, id: CardID) -> Result<()> {
    conn.lock().unwrap().execute(
        "INSERT INTO pending_cards (id, position) VALUES (?1, ?2)",
        params![id, 1],
    )?;
    Ok(())
}
