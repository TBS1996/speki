use crate::utils::aliases::*;
use rand::prelude::*;
use rusqlite::{params, Connection, Result};
use std::sync::MutexGuard;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use super::fetch::*;
use super::insert::*;

pub fn update_card_question(conn: &Arc<Mutex<Connection>>, id: u32, name: String) -> Result<()> {
    let mut stmt = conn.prepare("UPDATE cards SET question = ? WHERE id = ?")?;
    stmt.execute(params![name, id])?;
    Ok(())
}

pub fn is_table_empty(conn: &Arc<Mutex<Connection>>, table_name: String) -> bool {
    let mut stmt = conn.prepare("SELECT EXISTS (SELECT 1 FROM my_table)")?;
    stmt.execute(params![name, id])?;
    Ok(())
}
