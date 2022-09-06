use crate::utils::widgets::cardlist::CardItem;
use super::sql::insert::new_incread;
use super::sql::fetch::{get_incread, load_extracts, load_cloze_cards};
use super::card::Card;
use super::aliases::*;
use super::widgets::textinput::Field;
use rusqlite::Connection;
use crate::utils::statelist::StatefulList;
use crossterm::event::KeyCode;
use crate::utils::sql::update::update_inc_text;


#[derive(Clone)]
pub struct IncListItem{
    pub text: String,
    pub id: IncID,
}
pub struct IncRead{
    pub id: IncID,
    pub parent: IncID,
    pub topic: TopicID,
    pub source: Field,
    pub extracts: StatefulList<IncListItem>,
    pub clozes: StatefulList<CardItem>,
    pub isactive: bool,
}



impl IncRead{


    pub fn new(conn: &Connection, id: IncID) -> Self{
         get_incread(conn, id).unwrap()
    }
    

    pub fn extract(&mut self, conn: &Connection){
        if let Some(extract) = self.source.return_selection(){
            new_incread(conn, self.id, self.topic, extract, true).unwrap();
            self.extracts = StatefulList::with_items(load_extracts(conn, self.id).unwrap());
        }
    }
    pub fn cloze(&mut self, conn: &Connection){
        if let Some(cloze) = self.source.return_selection(){
            let mut question = self.source.text.clone();
            question = question.replace(&cloze, "[...]");
            let answer = cloze;
            Card::save_new_card(conn, question, answer, self.topic, self.id);
            self.clozes = StatefulList::with_items(load_cloze_cards(conn, self.id).unwrap());
        }
    }

    pub fn keyhandler(&mut self, conn: &Connection, key: KeyCode){
        match key {
            KeyCode::F(1) => {
                self.extract(conn);
                self.source.deselect();
            },
            KeyCode::F(2) => {
                self.cloze(conn);
                self.source.deselect();
            },
            KeyCode::Esc => {
        //        self
            },
            key => self.source.keyhandler(key),

        }
    }
    pub fn update_text(&self, conn: &Connection){
        let text = self.source.text.clone();
        update_inc_text(conn, text, self.id).unwrap();
            
    }
    

}





