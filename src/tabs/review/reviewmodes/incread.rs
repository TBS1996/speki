use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tui::{layout::Rect, backend::Backend, Frame, style::Style};

use crate::{
    tabs::review::logic::Action,
    utils::{aliases::{IncID, CardID}, incread::{IncRead, IncListItem}, misc::{View, split_leftright_by_percent, split_updown_by_percent}, statelist::StatefulList},
    MyKey, app::AppData, MyType,
};

pub struct IncMode {
    pub source: IncRead,
    view: View,
}


impl IncMode {
    fn set_selection(&mut self, area: Rect){
        let mainvec = split_leftright_by_percent([75, 15], area);
        let (editing, rightside) = (mainvec[0], mainvec[1]);
        let rightvec = split_updown_by_percent([10, 40, 40], rightside);

        self.view.areas.insert("editing", editing);
        self.view.areas.insert("source", rightvec[0]);
        self.view.areas.insert("extracts", rightvec[1]);
        self.view.areas.insert("clozes", rightvec[2]);
    }

    pub fn new(appdata: &AppData, id: CardID) -> Self{
        let source = IncRead::new(&appdata.conn, id);
        let view = View::default();
        Self {
            source, 
            view,
        }
    }

 pub fn render(&mut self, f: &mut Frame<MyType>, _conn: &Arc<Mutex<Connection>>, area: Rect)
    {
        self.set_selection(area);
        self.source.source.render(f, self.view.get_area("editing"), self.view.name_selected("editing"));
        self.source.extracts.render(f, self.view.get_area("extracts"), self.view.name_selected("extracts"), "Extracts", Style::default());
        self.source.clozes.render(f, self.view.get_area("clozes"), self.view.name_selected("clozes"), "Clozes", Style::default());
    }


    
    pub fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey, action: &mut Action) {
        use MyKey::*;

        if let MyKey::Nav(dir) = key {
            self.view.navigate(dir);
            return;
        }

        match key {
            KeyPress(pos) => self.view.cursor = pos,
            Alt('d') => {
                *action = Action::IncDone(
                    self.source.source.return_text(),
                    self.source.id,
                    self.source.source.cursor.clone(),
                )
            }
            Alt('s') => {
                *action = Action::IncNext(
                    self.source.source.return_text(),
                    self.source.id,
                    self.source.source.cursor.clone(),
                )
            }
            Alt('a') if self.view.name_selected("editing") => *action = Action::AddChild(self.source.id),
            key if self.view.name_selected("extracts") => self.source.extracts.keyhandler(key),
            key if self.view.name_selected("clozes") => self.source.clozes.keyhandler(key),
            key if self.view.name_selected("editing") => self.source.keyhandler(conn, key),
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

                "#
        .to_string()
    }
}
