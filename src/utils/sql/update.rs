use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::{params, Connection, Result};
use crate::utils::aliases::*;
use rand::prelude::*;

use super::fetch::*;
use super::insert::*;


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

pub fn update_strength(conn: &Connection, id: CardID, strength: f32) -> Result<()> {
    let mut stmt = conn.prepare("UPDATE finished_cards SET strength = ? WHERE id = ?")?;
    stmt.execute(params![strength, id])?;
    Ok(())
}

pub fn set_stability(conn: &Connection, id: CardID, stability: f32) -> Result<()>{
    
    let mut stmt = conn.prepare("UPDATE finished_cards SET stability = ? WHERE id = ?")?; 
    stmt.execute(params![stability, id])?;
    Ok(())
}




pub fn set_suspended(conn: &Connection, id: CardID, suspended: bool) -> Result<()>{
    let mut stmt = conn.prepare("UPDATE cards SET suspended = ? WHERE id = ?")?;
    stmt.execute(params![suspended, id])?;
    Ok(())
}

pub fn set_resolved(conn: &Connection, id: CardID, resolved: bool) -> Result<()>{
    let mut stmt = conn.prepare("UPDATE cards SET resolved = ? WHERE id = ?")?;
    stmt.execute(params![resolved, id])?;
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

pub fn update_card_source(conn: &Connection, cardid: CardID, incid: IncID) -> Result<()>{
    let mut stmt = conn.prepare("UPDATE cards SET source = ? WHERE id = ?")?;
    stmt.execute(params![incid, cardid])?;
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

pub fn double_skip_duration(conn: &Connection, id: CardID) -> Result<()>{
    let mut rng = rand::thread_rng();
    let mut y: f64 = rng.gen();
    y += 0.5; // y is now between 0.5 and 1.5
    let skipduration = get_skipduration(conn, id).unwrap();
    let new_skipduration = std::cmp::max((skipduration as f64 * y * 2.0) as u32, 2);
    let mut stmt = conn.prepare("UPDATE unfinished_cards SET skipduration = ? WHERE id = ?")?;
    stmt.execute(params![new_skipduration, id])?;
    update_skiptime(conn, id).unwrap();
    Ok(())
}


pub fn update_skiptime(conn: &Connection, id: CardID) -> Result<()>{
    let unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
    let mut stmt = conn.prepare("UPDATE unfinished_cards SET skiptime = ? WHERE id = ?")?;
    stmt.execute(params![unix, id])?;
    Ok(())
}
