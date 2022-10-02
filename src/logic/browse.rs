use crate::utils::{
    statelist::StatefulList,    //   StatefulList,
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};


pub enum BrowseCursor{
    Filtered,
    Selected,
}

pub struct Browse{
    pub filtered: StatefulList<u32>,
    pub selected: StatefulList<u32>,
    pub cursor: BrowseCursor,

}

impl Browse {
    pub fn new(conn: &Arc<Mutex<Connection>>) -> Self{ 
        Browse {
            filtered: StatefulList::<u32>::load_cards(conn),
            selected: StatefulList::<u32>::load_empty(),
            cursor: BrowseCursor::Filtered,
        }
    }

    pub fn select(&mut self){
        if let Some(index) = self.filtered.state.selected(){
            let card = self.filtered.items.remove(index);
            self.selected.items.push(card);

            if index == self.filtered.items.len(){
                self.filtered.previous()
            }

            if self.filtered.items.len() == 0 {
                self.filtered.state.select(None);
            }
        }
    }
    
    pub fn deselect(&mut self){
        if let Some(index) = self.selected.state.selected(){
            let card = self.selected.items.remove(index);
            self.filtered.items.push(card);

            if index == self.selected.items.len(){
                self.selected.previous();
            }

            if self.selected.items.len() == 0 {
                self.selected.state.select(None);
            }
        }
    }
}

