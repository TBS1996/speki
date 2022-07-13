
use rusqlite::{params, Connection, Result};
use crate::utils::card::Card;


pub fn update_strength(card: &Card, strength: f32) -> Result<()> {
    let conn = Connection::open("dbflash.db")?;
    let mut stmt = conn.prepare("UPDATE cards SET strength = ? WHERE id = ?")?;
    stmt.execute(params![strength, card.card_id])?;
    Ok(())
}


pub fn update_stability(card: Card) -> Result<()> {
    let conn = Connection::open("dbflash.db")?;
    let stability = card.stability.clone();
    let mut stmt = conn.prepare("UPDATE cards SET stability = ? WHERE id = ?")?;
    stmt.execute(params![stability, card.card_id])?;
    Ok(())
}



