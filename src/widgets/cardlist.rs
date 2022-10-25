use std::fmt;
use std::fmt::Display;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::utils::aliases::*;
use crate::utils::sql::fetch::fetch_question;
use crate::utils::statelist::KeyHandler;

#[derive(Debug, Clone)]
pub struct CardItem {
    pub question: String,
    pub id: CardID,
}

impl CardItem {
    pub fn from_id(conn: &Arc<Mutex<Connection>>, id: CardID) -> Self{
        Self {
            question: fetch_question(conn, id),
            id,
        }
    }
}

impl KeyHandler for CardItem {}

impl Display for CardItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.question)
    }
}
