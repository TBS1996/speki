use rusqlite::{params, Connection, Result};
use crate::utils::card::Card;


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



