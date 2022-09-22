use rusqlite::{params, Connection, Result};
use crate::utils::card::{Card, Review, CardType};//, Status, Topic, Review}
use crate::utils::aliases::*;
use std::time::{SystemTime, UNIX_EPOCH};


pub fn save_card(conn: &Connection, card: Card)-> Result<()>{
    let time_added = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;

    let cardtype = match card.cardtype{
        CardType::Pending    => 0,
        CardType::Unfinished => 1,
        CardType::Finished   => 2,
    };

    conn.execute(
        "INSERT INTO cards (question, answer, cardtype, suspended, resolved, topic, source) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
        card.question, 
        card.answer, 
        cardtype,
        card.suspended, 
        card.resolved, 
        card.topic, 
        card.source,
    //    time_added,
        ],
    )?;

    let id = conn.last_insert_rowid() as u32;


    match card.cardtype{
        CardType::Pending => new_pending(conn, id)?,
        CardType::Unfinished => new_unfinished(conn, id)?,
        CardType::Finished => new_finished(conn, id)?,
    };

    Ok(())
}

pub fn update_both(conn: &Connection, dependent: u32, dependency: u32) -> Result<()>{
    conn.execute(
        "INSERT INTO dependencies (dependent, dependency) VALUES (?1, ?2)",
        params![dependent, dependency],
    )?;
    Ok(())
}

pub fn revlog_new(conn: &Connection, card_id: u32, review: Review) -> Result<()> {
    conn.execute(
        "INSERT INTO revlog (unix, cid, grade, qtime, atime) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![review.date, card_id, review.grade as u32, review.answertime, -1],
    )?;
    Ok(())
}


pub fn new_topic(conn: &Connection, name: String, parent: u32, pos: u32) -> Result<()>{
    conn.execute(
        "INSERT INTO topics (name, parent, relpos) VALUES (?1, ?2, ?3)",
        params![name, parent, pos],
    )?;
    Ok(())
}


pub fn new_incread(conn: &Connection, parent: u32, topic: u32, source: String, isactive: bool) -> Result<()>{
    conn.execute(
        "INSERT INTO incread (parent, topic, source, active) VALUES (?1, ?2, ?3, ?4)",
        params![parent, topic, source, isactive],
    )?;
    Ok(())
}

pub fn new_finished(conn: &Connection, id: CardID) -> Result<()>{
    conn.execute(
        "INSERT INTO finished_cards (id, strength, stability) VALUES (?1, ?2, ?3)",
        params![id, 1.0, 1.0],
    )?;
    Ok(())
}

pub fn new_unfinished(conn: &Connection, id: CardID) -> Result<()>{
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
    conn.execute(
        "INSERT INTO unfinished_cards (id, skiptime, skipduration) VALUES (?1, ?2, ?3)",
        params![id, now, 1],
    )?;
    Ok(())
}
pub fn new_pending(conn: &Connection, id: CardID) -> Result<()>{
    conn.execute(
        "INSERT INTO pending_cards (id, position) VALUES (?1, ?2)",
        params![id, 1],
    )?;
    Ok(())
}
