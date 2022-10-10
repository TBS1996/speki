use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::logic::add_card::{NewCard, TextSelect};
use crate::MyKey;

impl NewCard {
    pub fn keyhandler(&mut self,conn: &Arc<Mutex<Connection>>, key: MyKey) {
        use MyKey::*;
        use TextSelect::*;
        match (&self.selection, key) {
            (_, Nav(dir)) => self.navigate(dir),
            (_, Alt('f')) => self.submit_card(conn, true),
            (_, Alt('u')) => self.submit_card(conn, false),
            (Question, key) => self.question.keyhandler(key),
            (Answer, key) => self.answer.keyhandler(key),
            (Topic, key) => self.topics.keyhandler(key, conn),
            (_, _) => {}
        }
    }
}
