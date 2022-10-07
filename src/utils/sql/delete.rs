use rusqlite::{Connection, params, Result};
use std::sync::{Mutex, Arc};
use crate::utils::aliases::*;

pub fn delete_topic(conn: &Arc<Mutex<Connection>>, id: TopicID) -> Result<()> {
    conn
        .lock()
        .unwrap()
        .prepare("delete from topics where id = ?")?
        .execute(params![id])?;
    Ok(())
}


pub fn remove_unfinished(conn: &Arc<Mutex<Connection>>, id: CardID) -> Result<()> {
    conn
        .lock()
        .unwrap()
        .prepare("delete from unfinished_cards where id = ?")?
        .execute(params![id])?;
    Ok(())
}


pub fn remove_pending(conn: &Arc<Mutex<Connection>>, id: CardID) -> Result<()> {
    conn
        .lock()
        .unwrap()
        .prepare("delete from pending_cards where id = ?")?
        .execute(params![id])?;
    Ok(())
}


pub fn remove_card(conn: &Arc<Mutex<Connection>>, id: CardID) -> Result<()> {
    conn
        .lock()
        .unwrap()
        .prepare("delete from cards where id = ?")?
        .execute(params![id])?;
   conn
        .lock()
        .unwrap()
        .prepare("delete from pending_cards where id = ?")?
        .execute(params![id])?;
   conn
        .lock()
        .unwrap()
        .prepare("delete from unfinished_cards where id = ?")?
        .execute(params![id])?;
   conn
        .lock()
        .unwrap()
        .prepare("delete from finished_cards where id = ?")?
        .execute(params![id])?;
    Ok(())
}
