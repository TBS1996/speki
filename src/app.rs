use rusqlite::Connection;
use crossterm::event::KeyCode;

use crate::{logic::{
    review::ReviewList,
    browse::Browse,
    add_card::{NewCard, DepState}, incread::MainInc,
}, utils::widgets::find_card::{FindCardWidget, FindCardStatus}};
use crate::events::{
    review::review_event,
    browse::browse_event,
    add_card::add_card_event,
    import::main_port,
    incread::main_inc,
};







pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}


impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0}
    }
    pub fn next(&mut self) {
        if self.index < self.titles.len() - 1 {
            self.index += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }
}

/* 

Architecture idea: 
each tab has a trait with a keyhandler and a render function. 
perhaps return a custom result option if needed 

perhaps the tabs are a vector of tabs, you can move them around and close and open and such


perhaps popup is a simple option 

option type is one that has the same render and keyhandler trait as above 

  */

pub enum PopUp{
    None,
    CardSelecter(FindCardWidget),
}



pub struct App<'a> {
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub conn: Connection,
    pub prev_key: KeyCode,
    pub review: ReviewList,
    pub add_card: NewCard,
    pub browse: Browse,
    pub incread: MainInc,
    pub popup: PopUp,
}

//use crate::logic::incread::MainInc;

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let conn    = Connection::open("dbflash.db").expect("Failed to connect to database.");
        let revlist = ReviewList::new(&conn);
        let browse  = Browse::new(&conn);
        let mut addcards =  NewCard::new(&conn, DepState::None);
        addcards.topics.next();
        let incread = MainInc::new(&conn);
        let popup = PopUp::None;


        App {
            should_quit: false,
            tabs: TabsState::new(vec!["Review", "Add card", "Browse cards 🦀", "Incremental reading"]),
            conn,
            prev_key: KeyCode::Null,
            review: revlist,
            add_card: addcards,
            browse,
            incread,
            popup,
        }
    }


    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_tick(&mut self) {}

    pub fn handle_key(&mut self, key: KeyCode) {
        use PopUp::*;

        match &mut self.popup{
            None => {
                if KeyCode::Tab == key {self.on_right()};
                if KeyCode::BackTab == key {self.on_left()};
                let keyclone = key.clone();
                match self.tabs.index {
                    0 => review_event(self, key),
                    1 => add_card_event(self, key),
                    2 => browse_event(self, key),
                    3 => main_port(self, key),
                    4 => main_inc(self, key),
                    _ => {},
                }
                self.prev_key = keyclone;
            },
            CardSelecter(findcard) => {
                findcard.keyhandler(&self.conn, key);
                if let FindCardStatus::Finished = findcard.status{
                    self.popup = None;
                    return;
                }
            },
            
        }

    }


}

