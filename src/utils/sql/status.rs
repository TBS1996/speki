use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::{params, Connection, Result};
use crate::utils::aliases::*;
use rand::prelude::*;
use std::sync::MutexGuard;
use std::sync::{Mutex, Arc};

use super::fetch::*;
use super::insert::*;


pub fn update_card_question(conn: &Arc<Mutex<Connection>>, id: u32, name: String) -> Result<()>{
    let mut stmt = conn.prepare("UPDATE cards SET question = ? WHERE id = ?")?;
    stmt.execute(params![name, id])?;
    Ok(())
}



