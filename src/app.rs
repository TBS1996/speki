use rusqlite::Connection;


use crate::{
    logic::{
        review::ReviewList,
        browse::Browse,
        add_card::{NewCard, DepState}, incread::MainInc,
    }, 
    SpekiPaths};

use crate::events::browse::browse_event;


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


use crate::logic::import::Importer;
use std::sync::{Arc, Mutex};





pub struct App<'a> {
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub conn: Arc<Mutex<Connection>>,
    pub review: ReviewList,
    pub add_card: NewCard,
    pub browse: Browse,
    pub incread: MainInc,
    pub importer: Importer,
    pub audio: rodio::OutputStream,
    pub audio_handle: rodio::OutputStreamHandle,
    pub display_help: bool,
    pub paths: SpekiPaths,
}

impl<'a> App<'a> {
    pub fn new(display_help: bool, paths: SpekiPaths) -> App<'a> {
        let conn    = Connection::open(&paths.database).expect("Failed to connect to database.");
        let conn = Arc::new(Mutex::new(conn));
        let (audio, audio_handle) = rodio::OutputStream::try_default().unwrap();
        let revlist = ReviewList::new(&conn, &audio_handle);
        let browse  = Browse::new(&conn);
        let addcards =  NewCard::new(&conn, DepState::None);
        let incread = MainInc::new(&conn);
        let importer = Importer::new(&conn);

        App {
            should_quit: false,
            tabs: TabsState::new(vec!["Review", "Add card", "Incremental reading", "import"]),  
            conn,
            review: revlist,
            add_card: addcards,
            browse,
            incread,
            importer,
            audio,
            audio_handle,
            display_help,
            paths,
        }
    }


    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }


    pub fn keyhandler(&mut self, key: MyKey) {
        match key {
            MyKey::Tab => self.on_right(),
            MyKey::BackTab => self.on_left(),
            MyKey::F(1) => self.display_help = !self.display_help,
            MyKey::Alt('q') => self.should_quit = true,
            _ => {},
        };
        match self.tabs.index {
            0 => self.review.keyhandler(&self.conn, key, &self.audio_handle),
            1 => self.add_card.keyhandler(&self.conn, key),
            2 => self.incread.keyhandler(&self.conn, key),
            3 => self.importer.keyhandler(&self.conn, key, &self.audio_handle, &self.paths),
            4 => browse_event(self,   key),
            _ => {},
        }
    }
}

use crate::MyKey;

