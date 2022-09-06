use rusqlite::Connection;
use crossterm::event::KeyCode;

use crate::{logic::{
    review::ReviewList,
    browse::Browse,
    add_card::{NewCard, DepState}, incread::MainInc,
}, utils::widgets::find_card::FindCardWidget};
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


// structs where sometimes you must choose a card can have that findcard widget in their struct 
// itll be none normally. if you need to select a card, make the selection enum equal to findcard,
// when you click select it will become a Some(Card) value, the next call it will extract that
// value for its purpose and turn it back to none afterwards
//
// hmm can enum be generic or something
//
// maybe a trait that is something like "selected card" so that when you bring certain widgets that
// require a card it can be automatically configured with that trait
//

pub struct App<'a> {
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub conn: Connection,
    pub prev_key: KeyCode,
    pub review: ReviewList,
    pub add_card: NewCard,
    pub browse: Browse,
    pub cardfinder: FindCardWidget,
    pub incread: MainInc,
}

//use crate::logic::incread::MainInc;

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let conn    = Connection::open("dbflash.db").expect("Failed to connect to database.");
        let revlist = ReviewList::new(&conn);
        let browse  = Browse::new(&conn);
        let cardfinder = FindCardWidget::new(&conn, "find a card!!".to_string());
        let mut addcards =  NewCard::new(&conn, DepState::None);
        addcards.topics.next();
        let incread = MainInc::new(&conn);


        App {
            should_quit: false,
            tabs: TabsState::new(vec!["Review", "Add card", "Browse cards 🦀", "import", "Incremental reading"]),
            conn,
            prev_key: KeyCode::Null,
            review: revlist,
            add_card: addcards,
            browse,
            cardfinder,
            incread,
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
    }
}

