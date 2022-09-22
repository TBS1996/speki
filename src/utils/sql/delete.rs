use rusqlite::{Connection, params, Result};


pub fn delete_topic(conn: &Connection, id: u32) -> Result<()> {
    let mut stmt = conn.prepare("delete from topics where id = ?")?;
    stmt.execute(params![id])?;
    Ok(())
}




pub fn remove_unfinished(conn: &Connection, id: u32) -> Result<()> {
    let mut stmt = conn.prepare("delete from unfinished_cards where id = ?")?;
    stmt.execute(params![id])?;
    Ok(())
}
