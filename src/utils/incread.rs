use crate::utils::widgets::cardlist::CardItem;
use super::sql::insert::new_incread;
use super::sql::fetch::{get_incread, load_extracts, load_cloze_cards};
use super::card::Card;
use super::aliases::*;
use super::widgets::textinput::Field;
use rusqlite::Connection;
use crate::utils::statelist::StatefulList;
use crate::MyKey;
use crate::utils::sql::update::update_inc_text;
use crate::utils::widgets::textinput::Mode;


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
            let mut question = self.source.return_text();
            question = question.replace(&cloze, "[...]");
            let answer = cloze;
            Card::save_new_card(conn, question, answer, self.topic, self.id, true);
            self.clozes = StatefulList::with_items(load_cloze_cards(conn, self.id).unwrap());
        }
    }


    pub fn keyhandler(&mut self, conn: &Connection, key: MyKey){
        match key {
            MyKey::Alt('x') => {
                self.extract(conn);
                self.source.set_normal_mode();
            },
            MyKey::Alt('z') => {
                self.cloze(conn);
                self.source.set_normal_mode();
            },
            MyKey::Esc => {
        //        self
            },
            key => self.source.keyhandler(key),

        }
    }
    pub fn update_text(&self, conn: &Connection){
        let text = self.source.return_text();
        update_inc_text(conn, text, self.id).unwrap();
    }
}





