use tui::widgets::ListState;
use crate::utils::sql::fetch::load_cards;
use rusqlite::Connection;






#[derive(Clone)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}


impl<T> StatefulList<T> {

    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }
    pub fn new() -> StatefulList<T> {
        let items = Vec::<T>::new();
        StatefulList {
            state: ListState::default(),
            items,
        }
    }
    
    pub fn load_cards(conn: &Connection) -> StatefulList<u32> {
        let cardvec = load_cards(conn).unwrap();
        let mut items = Vec::<u32>::new();
        for card in cardvec{
            items.push(card.card_id);
        }
        
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn load_empty() -> StatefulList<u32> {
        let items = Vec::<u32>::new();
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        if self.items.len() == 0 {return};

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                   self.items.len() - 1 
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.len() == 0 {return};

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                    }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}




