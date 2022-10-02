use rusqlite::{Connection, params, Result};
use std::sync::MutexGuard;
use std::sync::{Mutex, Arc};


pub fn delete_topic(conn: &Arc<Mutex<Connection>>, id: u32) -> Result<()> {
    conn
        .lock()
        .unwrap()
        .prepare("delete from topics where id = ?")?
        .execute(params![id])?;
    Ok(())
}




pub fn remove_unfinished(conn: &Arc<Mutex<Connection>>, id: u32) -> Result<()> {
    conn
        .lock()
        .unwrap()
        .prepare("delete from unfinished_cards where id = ?")?
        .execute(params![id])?;
    Ok(())
}


pub fn remove_pending(conn: &Arc<Mutex<Connection>>, id: u32) -> Result<()> {
    conn
        .lock()
        .unwrap()
        .prepare("delete from pending_cards where id = ?")?
        .execute(params![id])?;
    Ok(())
}
