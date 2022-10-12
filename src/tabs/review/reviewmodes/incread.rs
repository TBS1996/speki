use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::{
    tabs::review::logic::Action,
    utils::{aliases::IncID, incread::IncRead},
    MyKey,
};


pub struct IncMode {
    pub id: IncID,
    pub source: IncRead,
    pub selection: IncSelection,
}

pub enum IncSelection {
    Source,
    Clozes,
    Extracts,
}





impl IncMode {
    pub fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey, action: &mut Action) {
        use IncSelection::*;
        use MyKey::*;

        if let MyKey::Nav(dir) = &key {
            self.inc_nav(dir);
            return;
        }
        match (&self.selection, key) {
            (_, Alt('d')) => {
                *action = Action::IncDone(
                    self.source.source.return_text(),
                    self.id,
                    self.source.source.cursor.clone(),
                )
            }
            (_, Alt('s')) => {
                *action = Action::IncNext(
                    self.source.source.return_text(),
                    self.id,
                    self.source.source.cursor.clone(),
                )
            }
            (Source, Alt('a')) => *action = Action::AddChild(self.id),
            (Source, key) => self.source.keyhandler(conn, key),
            (_, _) => {}
        }
    }
    fn inc_nav(&mut self, dir: &crate::Direction) {
        use crate::Direction::*;
        use IncSelection::*;
        match (&self.selection, dir) {
            (Source, Right) => self.selection = Extracts,

            (Clozes, Up) => self.selection = Extracts,
            (Clozes, Left) => self.selection = Source,

            (Extracts, Left) => self.selection = Source,
            (Extracts, Down) => self.selection = Clozes,
            _ => {}
        }
    }


    pub fn get_manual(&self) -> String {
        r#"
            
        Mark text as done: Alt+d
        skip text: Alt+s
        make extract (visual mode): Alt+x 
        make cloze (visual mode): Alt+z
        add child card(in text widget): Alt+a

                "#.to_string()
    }


}
