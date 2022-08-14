use rusqlite::{params, Connection, Result};
use crate::utils::card::{Card, Review};//, Status, Topic, Review}



pub fn save_card(conn: &Connection, somecard: Card)-> Result<()>{

    conn.execute(
        "INSERT INTO cards (question, answer, strength, stability, topic, initiated, complete, resolved, suspended, gain) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![somecard.question, somecard.answer, somecard.strength, somecard.stability, somecard.topic, somecard.status.initiated, somecard.status.complete, somecard.status.resolved, somecard.status.suspended, -1.0],
    )?;

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


pub fn new_topic(conn: &Connection, name: String, parent: u32) -> Result<()>{
    conn.execute(
        "INSERT INTO topics (name, parent) VALUES (?1, ?2)",
        params![name, parent],
    )?;
    Ok(())
}
