
use crate::utils::sql::fetch::load_cards;
use tui::widgets::ListState;

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
    
    pub fn load_cards() -> StatefulList<u32> {
        let cardvec = load_cards().unwrap();
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
    pub fn new() -> Self{ 
        Browse {
            filtered: StatefulList::<u32>::load_cards(),
            selected: StatefulList::<u32>::load_empty(),
            cursor: BrowseCursor::Filtered,
        }
    }

    pub fn select(&mut self){
        if let Some(index) = self.filtered.state.selected(){
            let card = self.filtered.items.remove(index);
            self.selected.items.push(card);

            if index == self.filtered.items.len(){
                self.filtered.previous();
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

