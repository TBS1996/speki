use super::fetch::*;
use crate::utils::aliases::*;
use crate::utils::card::CardType;
use crate::widgets::textinput::CursorPos;
use rand::prelude::*;
use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn update_card_question(conn: &Arc<Mutex<Connection>>, id: u32, name: String) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE cards SET question = ? WHERE id = ?")?
        .execute(params![name, id])?;
    Ok(())
}
pub fn update_card_answer(conn: &Arc<Mutex<Connection>>, id: u32, name: String) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE cards SET answer = ? WHERE id = ?")?
        .execute(params![name, id])?;
    Ok(())
}

pub fn update_strength(conn: &Arc<Mutex<Connection>>, id: CardID, strength: f32) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE finished_cards SET strength = ? WHERE id = ?")
        .unwrap()
        .execute(params![strength, id])?;
    Ok(())
}

pub fn set_stability(conn: &Arc<Mutex<Connection>>, id: CardID, stability: f32) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE finished_cards SET stability = ? WHERE id = ?")
        .unwrap()
        .execute(params![stability, id])?;
    Ok(())
}

pub fn set_cardtype(conn: &Arc<Mutex<Connection>>, id: CardID, cardtype: CardType) -> Result<()> {
    let cardtype = match cardtype {
        CardType::Pending => 0,
        CardType::Unfinished => 1,
        CardType::Finished => 2,
    };

    conn.lock()
        .unwrap()
        .prepare("UPDATE cards SET cardtype = ? WHERE id = ?")
        .unwrap()
        .execute(params![cardtype, id])?;
    Ok(())
}
pub fn set_suspended(conn: &Arc<Mutex<Connection>>, id: CardID, suspended: bool) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE cards SET suspended = ? WHERE id = ?")
        .unwrap()
        .execute(params![suspended, id])?;
    Ok(())
}

pub fn set_resolved(conn: &Arc<Mutex<Connection>>, id: CardID, resolved: bool) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE cards SET resolved = ? WHERE id = ?")
        .unwrap()
        .execute(params![resolved, id])?;
    Ok(())
}

pub fn update_topic_name(conn: &Arc<Mutex<Connection>>, id: u32, name: String) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE topics SET name = ? WHERE id = ?")
        .unwrap()
        .execute(params![name, id])?;
    Ok(())
}

pub fn update_topic_relpos(conn: &Arc<Mutex<Connection>>, id: u32, relpos: u32) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE topics SET relpos = ? WHERE id = ?")
        .unwrap()
        .execute(params![relpos, id])?;
    Ok(())
}

pub fn update_card_source(
    conn: &Arc<Mutex<Connection>>,
    cardid: CardID,
    incid: IncID,
) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE cards SET source = ? WHERE id = ?")
        .unwrap()
        .execute(params![incid, cardid])?;
    Ok(())
}
pub fn update_card_topic(
    conn: &Arc<Mutex<Connection>>,
    old_topic: u32,
    new_topic: u32,
) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE cards SET topic = ? WHERE topic = ?")
        .unwrap()
        .execute(params![new_topic, old_topic])?;
    Ok(())
}

pub fn update_topic_parent(conn: &Arc<Mutex<Connection>>, id: u32, parent: u32) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE topics SET parent = ? WHERE id = ?")
        .unwrap()
        .execute(params![parent, id])?;
    Ok(())
}

pub fn update_inc_text(
    conn: &Arc<Mutex<Connection>>,
    source: String,
    id: IncID,
    cursor: &CursorPos,
) -> Result<()> {
    let row = cursor.row;
    let column = cursor.column;

    conn.lock()
        .unwrap()
        .prepare("UPDATE incread SET source = ?1, row = ?2, column = ?3 WHERE id = ?4")
        .unwrap()
        .execute(params![source, row, column, id])?;
    Ok(())
}

pub fn update_inc_active(conn: &Arc<Mutex<Connection>>, id: IncID, active: bool) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE incread SET active = ? WHERE id = ?")
        .unwrap()
        .execute(params![active, id])?;
    Ok(())
}

pub fn double_skip_duration(conn: &Arc<Mutex<Connection>>, id: CardID) -> Result<()> {
    update_skiptime(conn, id).unwrap();
    let mut rng = rand::thread_rng();
    let mut y: f64 = rng.gen();
    y += 0.5; // y is now between 0.5 and 1.5
    let skipduration = get_skipduration(conn, id).unwrap();
    let new_skipduration = std::cmp::max((skipduration as f64 * y * 2.0) as u32, 2);
    conn.lock()
        .unwrap()
        .prepare("UPDATE unfinished_cards SET skipduration = ? WHERE id = ?")
        .unwrap()
        .execute(params![new_skipduration, id])?;
    Ok(())
}

pub fn update_skiptime(conn: &Arc<Mutex<Connection>>, id: CardID) -> Result<()> {
    let unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    conn.lock()
        .unwrap()
        .prepare("UPDATE unfinished_cards SET skiptime = ? WHERE id = ?")
        .unwrap()
        .execute(params![unix, id])?;
    Ok(())
}

pub fn double_inc_skip_duration(conn: &Arc<Mutex<Connection>>, id: IncID) -> Result<()> {
    update_inc_skiptime(conn, id).unwrap();
    let mut rng = rand::thread_rng();
    let mut y: f64 = rng.gen();
    y += 0.5; // y is now between 0.5 and 1.5
    let skipduration = get_inc_skipduration(conn, id).unwrap();
    let new_skipduration = std::cmp::max((skipduration as f64 * y * 2.0) as u32, 2);
    conn.lock()
        .unwrap()
        .prepare("UPDATE incread SET skipduration = ? WHERE id = ?")
        .unwrap()
        .execute(params![new_skipduration, id])?;
    Ok(())
}

pub fn update_inc_skiptime(conn: &Arc<Mutex<Connection>>, id: IncID) -> Result<()> {
    let unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    conn.lock()
        .unwrap()
        .prepare("UPDATE incread SET skiptime = ? WHERE id = ?")
        .unwrap()
        .execute(params![unix, id])?;
    Ok(())
}
