use rusqlite::Connection;


use crate::{
    logic::{
        review::ReviewList,
        browse::Browse,
        add_card::{NewCard, DepState}, incread::MainInc,
    }, 
    utils::widgets::find_card::FindCardWidget, tabs::Widget};
use crate::events::{
    review::review_event,
    browse::browse_event,
    add_card::add_card_event,
    incread::main_inc,
};

use crate::utils::misc::PopUpStatus;

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


ok wait

there should be a tab struct 
it keeps all the widgets in it,
and it handles navigation
and tab-specific functions

it can also handle popups, so that you can have  a popup on one tab but able to switch to other tabs




maybe widgets like topiclist and stuff should be mutable references so that they'll stay in sync
wait that would fuck up selection lol

*/

use crate::utils::widgets::newchild::AddChildWidget;
use crate::utils::widgets::filepicker::Directory;



pub enum PopUp{
    None,
    CardSelecter(FindCardWidget),
    AddChild(AddChildWidget),
}



pub struct App<'a> {
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub conn: Connection,
    pub review: ReviewList,
    pub add_card: NewCard,
    pub browse: Browse,
    pub incread: MainInc,
    pub debug: Directory,
    pub popup: PopUp,
}


impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let conn    = Connection::open("dbflash.db").expect("Failed to connect to database.");
        let revlist = ReviewList::new(&conn);
        let browse  = Browse::new(&conn);
        let mut addcards =  NewCard::new(&conn, DepState::None);
        addcards.topics.next();
        let incread = MainInc::new(&conn);
        let popup = PopUp::None;
        let debug = Directory::new();


        App {
            should_quit: false,
            tabs: TabsState::new(vec!["Review", "Add card", "Incremental reading"]),  //"Browse cards 🦀", "Incremental reading", "debug"]),
            conn,
            review: revlist,
            add_card: addcards,
            browse,
            incread,
            debug, 
            popup,
        }
    }


    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }


    pub fn handle_key(&mut self, key: MyKey) {
        use PopUp::*;


        if let MyKey::Alt('q') = key{
            self.should_quit = true;
            return;
        }

        match &mut self.popup{
            None => {
                match key {
                    MyKey::Tab => self.on_right(),
                    MyKey::BackTab => self.on_left(),
                    _ => {},
                };
                 

                match self.tabs.index {
                    0 => review_event(self,   key),
                    1 => add_card_event(self, key),
                    3 => browse_event(self,   key),
                    2 => main_inc(self,       key),
                    4 => {
                        self.debug.keyhandler(key);
                    },
                    _ => {},
                }
            },
            AddChild(addchild) => {
                addchild.keyhandler(&self.conn, key);
                if let PopUpStatus::Finished = addchild.status{
                    self.popup = None;
                    return;
                }
            },
            CardSelecter(findcard) => {
                findcard.keyhandler(&self.conn, key);
                if let PopUpStatus::Finished = findcard.status{
                    self.popup = None;
                    return;
                }
            },
        }
    }
}

use crate::MyKey;
