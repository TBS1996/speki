use crate::Direction;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{UNIX_EPOCH, SystemTime};
use std::{fs::File, io::Read};
use crate::MyKey;
use crate::tabs::Widget;
use crate::utils::widgets::button::draw_button;
use crate::utils::widgets::message_box::draw_message;
use crate::utils::widgets::textinput::Field;
use crate::utils::widgets::topics::TopicList;
use crate::utils::{
    card::{Card, Review, Status},
    sql::insert::save_card,

};
use tui::widgets::ListState;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction::{Vertical, Horizontal}, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, ListItem, List},
    text::Spans,
    Frame,
};
use crate::utils::aliases::*;
use rusqlite::Connection;
use csv::StringRecord;
use anyhow::Result;
use crate::utils::card::CardType;
use crate::MyType;
use reqwest;
use std::fs;
use std::io;
use crate::utils::widgets::ankimporter::{Ankimporter, ShouldQuit};
use crate::utils::widgets::load_cards::{Template, LoadState};



use crate::utils::widgets::filepicker::{FilePicker, PickState};

enum Selection{
    Topics,
    Notes,
    Nonefile,
    Urlfile,
    Localfile,
    Preview,
    Qtemp,
    Atemp,
    ImportButton,
}


struct Selected{
    topics: bool,
    notes: bool,
    nonefile: bool,
    urlfile: bool,
    localfile: bool,
    preview: bool,
    qtemp: bool,
    atemp: bool,
    importbutton: bool,
}

impl Selected{
    fn from(selected: &Selection)-> Self{
        use Selection::*;
        let mut foo = Selected{
            topics: false,
            notes: false,
            nonefile: false,
            urlfile: false,
            localfile: false,
            preview: false,
            qtemp: false,
            atemp: false,
            importbutton: false,
        };
        match selected{
            Topics => foo.topics = true,
            Notes => foo.notes = true,
            Nonefile => foo.nonefile = true,
            Urlfile => foo.urlfile = true,
            Localfile => foo.localfile = true,
            Preview => foo.preview = true,
            Qtemp => foo.qtemp = true,
            Atemp => foo.atemp = true,
            ImportButton => foo.importbutton = true,
        };
        foo
    }
}



enum DeckSource{
    id(u32),
    path(PathBuf),
}



enum Menu{
    Main,
    Anki(Ankimporter),
    Local(FilePicker),
    LoadCards(Template),
    
}

//use crate::utils::widgets::filepicker::FilePicker;

pub struct Importer{
    topics: TopicList,
    selection: Selection,
    menu: Menu,
}

use crate::utils::widgets::list::list_widget;

impl Importer{
    pub fn new(conn: &Connection) -> Importer{
        let topics = TopicList::new(conn);
        let filepicker = FilePicker::new(["apkg".to_string()]);
        let menu = Menu::Local(filepicker);
        let selection = Selection::Nonefile;

        Importer{
            topics,
            selection,
            menu,
        }
    }


    /*
    */
    pub fn render(&mut self, f: &mut tui::Frame<MyType>, area: tui::layout::Rect) {
        let selection = Selected::from(&self.selection);

        match &mut self.menu{
            Menu::Main => {},
            Menu::Local(filesource) => {
                filesource.render(f, area);
            },
            Menu::Anki(ankimporter) => {},
            Menu::LoadCards(tmpl) => {
                tmpl.render(f, area);
            },
        }
    }




    pub fn keyhandler(&mut self, conn: &Connection, key: MyKey){
        use MyKey::*;
        use Selection::*;
        
        match &mut self.menu{
            Menu::Main => {},
            Menu::Anki(ankimporter) => {},
            Menu::Local(loc) => {
                match &loc.state{
                    PickState::Ongoing => loc.keyhandler(key),
                    PickState::ExitEarly => {
                        loc.state = PickState::Ongoing;

                    },
                    PickState::Fetch(path) => {
                        let template = Template::new(conn, path.to_path_buf());
                        self.menu = Menu::LoadCards(template);
                        },
                    };
                },
            Menu::LoadCards(tmpl) => {
                match tmpl.state{
                    LoadState::OnGoing => tmpl.keyhandler(conn, key),
                    LoadState::Finished => {},
                    LoadState::Importing(_) => {},
                }
            }


        
        }

        }
}
  










use std::io::prelude::*;
use serde_json::json;



