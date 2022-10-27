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

