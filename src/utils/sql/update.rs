use rusqlite::{params, Connection, Result};
use crate::utils::card::Card;
use crate::utils::aliases::*;

use super::fetch::fetch_card;


pub fn update_card_question(conn: &Connection, id: u32, name: String) -> Result<()>{
    let mut stmt = conn.prepare("UPDATE cards SET question = ? WHERE id = ?")?;
    stmt.execute(params![name, id])?;
    Ok(())
}
pub fn update_card_answer(conn: &Connection, id: u32, name: String) -> Result<()>{
    let mut stmt = conn.prepare("UPDATE cards SET answer = ? WHERE id = ?")?;
    stmt.execute(params![name, id])?;
    Ok(())
}

pub fn update_strength(conn: &Connection, card: &Card, strength: f32) -> Result<()> {
    let mut stmt = conn.prepare("UPDATE cards SET strength = ? WHERE id = ?")?;
    stmt.execute(params![strength, card.card_id])?;
    Ok(())
}


pub fn update_stability(conn: &Connection, card: Card) -> Result<()> {
    let stability = card.stability.clone();
    let mut stmt = conn.prepare("UPDATE cards SET stability = ? WHERE id = ?")?;
    stmt.execute(params![stability, card.card_id])?;
    Ok(())
}

pub fn activate_card(conn: &Connection, id: CardID) -> Result<()> {
    let mut stmt = conn.prepare("UPDATE cards SET initiated = 1 WHERE id = ?")?;
    stmt.execute(params![id])?;
    Ok(())
}
pub fn update_status(conn: &Connection, card: &Card) -> Result<()>{
    let initiated = card.status.initiated;
    let complete = card.status.complete;
    let resolved = card.status.resolved;
    let suspended = card.status.suspended;
    let card_id = card.card_id;

    let mut stmt = conn.prepare("UPDATE cards SET (initiated, complete, resolved, suspended) = (?,?,?,?) WHERE id = ?")?;
    stmt.execute(params![initiated, complete, resolved, suspended, card_id])?;
    Ok(())
}


pub fn update_topic_name(conn: &Connection, id: u32, name: String) -> Result<()>{
    let mut stmt = conn.prepare("UPDATE topics SET name = ? WHERE id = ?")?;
    stmt.execute(params![name, id])?;
    Ok(())
}


pub fn update_topic_relpos(conn: &Connection, id: u32, relpos: u32) -> Result<()>{
    let mut stmt = conn.prepare("UPDATE topics SET relpos = ? WHERE id = ?")?;
    stmt.execute(params![relpos, id])?;
    Ok(())
}

pub fn update_card_topic(conn: &Connection, old_topic: u32, new_topic: u32) -> Result<()>{
    let mut stmt = conn.prepare("UPDATE cards SET topic = ? WHERE topic = ?")?;
    stmt.execute(params![new_topic, old_topic])?;
    Ok(())
}

pub fn update_topic_parent(conn: &Connection, id: u32, parent: u32) -> Result<()>{
    let mut stmt = conn.prepare("UPDATE topics SET parent = ? WHERE id = ?")?;
    stmt.execute(params![parent, id])?;
    Ok(())
}

pub fn update_inc_text(conn: &Connection, source: String, id: IncID) -> Result<()>{
    let mut stmt = conn.prepare("UPDATE incread SET source = ? WHERE id = ?")?;
    stmt.execute(params![source, id])?;
    Ok(())
}

pub fn update_inc_active(conn: &Connection, id: IncID, active: bool) -> Result<()>{
    let mut stmt = conn.prepare("UPDATE incread SET active = ? WHERE id = ?")?;
    stmt.execute(params![active, id])?;
    Ok(())
}



pub fn double_skip_duration(conn: &Connection, id: IncID) -> Result<()>{
    let card = fetch_card(conn, id);
    let mut stmt = conn.prepare("UPDATE cards SET skipduration = ? WHERE id = ?")?;
    stmt.execute(params![card.skipduration * 2, id])?;
    Ok(())
}
